#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m::{asm::delay, peripheral::DWT};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use rtic::cyccnt::{Instant, U32Ext as _};
use stm32f1xx_hal::usb::{Peripheral, UsbBus, UsbBusType};
use stm32f1xx_hal::{adc, gpio, prelude::*, stm32::ADC1};
use usb_device::bus;
use usb_device::prelude::*;

pub struct PanelState {
    throttle0: u16,
    throttle1: u16,
    sw: u8,
}

#[allow(unused)]
pub mod hid {

    use usb_device::class_prelude::*;
    use usb_device::Result;

    const USB_CLASS_HID: u8 = 0x03;

    const USB_SUBCLASS_NONE: u8 = 0x00;
    const USB_SUBCLASS_BOOT: u8 = 0x01;

    const USB_INTERFACE_NONE: u8 = 0x00;
    const USB_INTERFACE_KEYBOARD: u8 = 0x01;
    const USB_INTERFACE_MOUSE: u8 = 0x02;

    const REQ_GET_REPORT: u8 = 0x01;
    const REQ_GET_IDLE: u8 = 0x02;
    const REQ_GET_PROTOCOL: u8 = 0x03;
    const REQ_SET_REPORT: u8 = 0x09;
    const REQ_SET_IDLE: u8 = 0x0a;
    const REQ_SET_PROTOCOL: u8 = 0x0b;

    const REPORT_DESCR: &[u8] = &[
        0x05, 0x01, // USAGE_PAGE (Generic Desktop)
        0x09, 0x04, // USAGE (Joystick)
        0xa1, 0x01, // COLLECTION (Application)
        0x05, 0x02, // USAGE_PAGE (Simulation)
        0x09, 0xbb, // USAGE (Throttle)
        0x16, 0x00, 0x80, /* Logical Minimum (-32768) */
        0x26, 0xff, 0x7f, /* Logical Maximum (32767) */
        0x95, 0x01, // REPORT_COUNT (1)
        0x75, 0x10, // REPORT_SIZE (16)
        0x81, 0x02, // INPUT (Data,Var,Abs)
        0x05, 0x01, // USAGE_PAGE (Generic Desktop)
        0x09, 0x32, // USAGE (Z)
        0x16, 0x00, 0x80, /* Logical Minimum (-32768) */
        0x26, 0xff, 0x7f, /* Logical Maximum (32767) */
        0x95, 0x01, // REPORT_COUNT (1)
        0x75, 0x10, // REPORT_SIZE (16)
        0x81, 0x02, // INPUT (Data,Var,Abs)
        0x05, 0x09, // USAGE_PAGE (Button)
        0x19, 0x01, // USAGE_MINIMUM(Button 1)
        0x29, 0x05, // USAGE_MAXIMUM(Button 5)
        0x95, 0x05, // REPORT_COUNT (5)
        0x75, 0x01, // REPORT_SIZE (1)
        0x15, 0x00, /* Logical Minimum (0) */
        0x25, 0x01, /* Logical Maximum (1) */
        0x81, 0x02, // INPUT (Data,Var,Abs)
        0x95, 0x03, // REPORT_COUNT (3)
        0x75, 0x01, // REPORT_SIZE (1)
        0x81, 0x01, // INPUT (Cnst,Arr,Abs)
        0xc0, // END_COLLECTION
    ];

    pub fn report(s: super::PanelState) -> [u8; 5] {
        [
            s.throttle1 as u8, // throttle 0
            (s.throttle1 >> 8) as u8,
            s.throttle0 as u8, // throttle 1
            (s.throttle0 >> 8) as u8,
            s.sw,
        ]
    }

    pub struct HIDClass<'a, B: UsbBus> {
        report_if: InterfaceNumber,
        report_ep: EndpointIn<'a, B>,
    }

    impl<B: UsbBus> HIDClass<'_, B> {
        /// Creates a new HIDClass with the provided UsbBus and max_packet_size in bytes. For
        /// full-speed devices, max_packet_size has to be one of 8, 16, 32 or 64.
        pub fn new(alloc: &UsbBusAllocator<B>) -> HIDClass<'_, B> {
            HIDClass {
                report_if: alloc.interface(),
                report_ep: alloc.interrupt(8, 10),
            }
        }

        pub fn write(&mut self, data: &[u8]) {
            self.report_ep.write(data).ok();
        }
    }

    impl<B: UsbBus> UsbClass<B> for HIDClass<'_, B> {
        fn get_configuration_descriptors(&self, writer: &mut DescriptorWriter) -> Result<()> {
            writer.interface(
                self.report_if,
                USB_CLASS_HID,
                USB_SUBCLASS_NONE,
                USB_INTERFACE_MOUSE,
            )?;

            let descr_len: u16 = REPORT_DESCR.len() as u16;
            writer.write(
                0x21,
                &[
                    0x01,                   // bcdHID
                    0x01,                   // bcdHID
                    0x00,                   // bContryCode
                    0x01,                   // bNumDescriptors
                    0x22,                   // bDescriptorType
                    descr_len as u8,        // wDescriptorLength
                    (descr_len >> 8) as u8, // wDescriptorLength
                ],
            )?;

            writer.endpoint(&self.report_ep)?;

            Ok(())
        }

        fn control_in(&mut self, xfer: ControlIn<B>) {
            let req = xfer.request();

            if req.request_type == control::RequestType::Standard {
                match (req.recipient, req.request) {
                    (control::Recipient::Interface, control::Request::GET_DESCRIPTOR) => {
                        let (dtype, _index) = req.descriptor_type_index();
                        if dtype == 0x21 {
                            // HID descriptor
                            cortex_m::asm::bkpt();
                            let descr_len: u16 = REPORT_DESCR.len() as u16;

                            // HID descriptor
                            let descr = &[
                                0x09,                   // length
                                0x21,                   // descriptor type
                                0x01,                   // bcdHID
                                0x01,                   // bcdHID
                                0x00,                   // bCountryCode
                                0x01,                   // bNumDescriptors
                                0x22,                   // bDescriptorType
                                descr_len as u8,        // wDescriptorLength
                                (descr_len >> 8) as u8, // wDescriptorLength
                            ];

                            xfer.accept_with(descr).ok();
                            return;
                        } else if dtype == 0x22 {
                            // Report descriptor
                            xfer.accept_with(REPORT_DESCR).ok();
                            return;
                        }
                    }
                    _ => {
                        return;
                    }
                };
            }

            if !(req.request_type == control::RequestType::Class
                && req.recipient == control::Recipient::Interface
                && req.index == u8::from(self.report_if) as u16)
            {
                return;
            }

            match req.request {
                REQ_GET_REPORT => {
                    // USB host requests for report
                    // I'm not sure what should we do here, so just send empty report
                    xfer.accept_with(&report(super::PanelState {
                        throttle0: 0,
                        throttle1: 0,
                        sw: 0,
                    }))
                    .ok();
                }
                _ => {
                    xfer.reject().ok();
                }
            }
        }

        fn control_out(&mut self, xfer: ControlOut<B>) {
            let req = xfer.request();

            if !(req.request_type == control::RequestType::Class
                && req.recipient == control::Recipient::Interface
                && req.index == u8::from(self.report_if) as u16)
            {
                return;
            }

            xfer.reject().ok();
        }
    }
}

use hid::HIDClass;

const PERIOD: u32 = 80_000;

#[rtic::app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBusType>,
        hid: HIDClass<'static, UsbBusType>,
        adc1: adc::Adc<ADC1>,
        th0: gpio::gpioa::PA1<gpio::Analog>,
        th1: gpio::gpioa::PA2<gpio::Analog>,
        sw0: gpio::gpioa::PA8<gpio::Input<gpio::PullUp>>,
        sw1: gpio::gpioa::PA9<gpio::Input<gpio::PullUp>>,
        sw2: gpio::gpioa::PA10<gpio::Input<gpio::PullUp>>,
        sw3: gpio::gpiob::PB14<gpio::Input<gpio::PullUp>>,
        sw4: gpio::gpiob::PB15<gpio::Input<gpio::PullUp>>,
    }

    #[init(schedule = [on_tick])]
    fn init(mut cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        cx.core.DCB.enable_trace();
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .adcclk(2.mhz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low().ok();
        delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        *USB_BUS = Some(UsbBus::new(usb));

        let hid = HIDClass::new(USB_BUS.as_ref().unwrap());
        let adc1 = adc::Adc::adc1(cx.device.ADC1, &mut rcc.apb2, clocks);
        let th0 = gpioa.pa1.into_analog(&mut gpioa.crl);
        let th1 = gpioa.pa2.into_analog(&mut gpioa.crl);
        let sw0 = gpioa.pa8.into_pull_up_input(&mut gpioa.crh);
        let sw1 = gpioa.pa9.into_pull_up_input(&mut gpioa.crh);
        let sw2 = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);
        let sw3 = gpiob.pb14.into_pull_up_input(&mut gpiob.crh);
        let sw4 = gpiob.pb15.into_pull_up_input(&mut gpiob.crh);

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x0483, 0x002a))
            .manufacturer("Ted's")
            .product("Cessna Knobox")
            .serial_number("001")
            .device_class(0)
            .build();

        cx.schedule.on_tick(cx.start + PERIOD.cycles()).ok();

        init::LateResources {
            usb_dev,
            hid,
            adc1,
            th0,
            th1,
            sw0,
            sw1,
            sw2,
            sw3,
            sw4,
        }
    }

    #[task(schedule = [on_tick], resources = [hid, adc1, th0, th1, sw0, sw1, sw2, sw3, sw4])]
    fn on_tick(mut cx: on_tick::Context) {
        cx.schedule.on_tick(Instant::now() + PERIOD.cycles()).ok();

        let hid = &mut cx.resources.hid;
        let adc1 = &mut cx.resources.adc1;
        let raw0: u32 = adc1.read(cx.resources.th0).unwrap();
        let raw1: u32 = adc1.read(cx.resources.th1).unwrap();
        let throttle0 = (raw0 * 32767 / 4095) as u16;
        let throttle1 = (raw1 * 32767 / 4095) as u16;
        hid.write(&hid::report(PanelState {
            throttle0,
            throttle1,
            sw: if cx.resources.sw0.is_low().unwrap() {
                1
            } else {
                0
            } | (if cx.resources.sw1.is_low().unwrap() {
                1
            } else {
                0
            }) << 1
                | (if cx.resources.sw2.is_low().unwrap() {
                    1
                } else {
                    0
                }) << 2
                | (if cx.resources.sw3.is_low().unwrap() {
                    1
                } else {
                    0
                }) << 3
                | (if cx.resources.sw4.is_low().unwrap() {
                    1
                } else {
                    0
                }) << 4,
        }));
    }

    #[task(binds=USB_HP_CAN_TX, resources = [usb_dev, hid])]
    fn usb_tx(mut cx: usb_tx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.hid);
    }

    #[task(binds=USB_LP_CAN_RX0, resources = [usb_dev, hid])]
    fn usb_rx(mut cx: usb_rx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.hid);
    }

    extern "C" {
        fn EXTI0();
    }
};

fn usb_poll<B: bus::UsbBus>(usb_dev: &mut UsbDevice<'static, B>, hid: &mut HIDClass<'static, B>) {
    if !usb_dev.poll(&mut [hid]) {
        return;
    }
}

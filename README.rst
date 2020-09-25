Ted's Cessna Knobox
===================

.. raw:: html

    <div align="center">
    <img src="https://raw.githubusercontent.com/Determinant/cessna-knobox/master/front.jpg" width="70%">
    <img src="https://raw.githubusercontent.com/Determinant/cessna-knobox/master/internal.jpg" width="70%">
    </div>


Materials
---------

- a friction-locked throttle knob (I used )
- a vernier-style mixer knob (I used )
- hardboard, 3.5mm in thickness
- M3 screws x4 for securing the potentiometers to the holders
- M2 screws x4 for securing the knob ends to the potentiometer sliders
- (M2.5/M3 screws + nuts) x4 for securing the holders to the enclosure
- you'll need to laster-cut the hardboard and 3D-print the holders/sliders

3D Printed Components
---------------------

- Holders: ``holder.stl``
- Sliders: ``slider1.stl``, ``slider2.stl``

Enclosure
---------

- Part 1: ``cessna-knobox-enclosure1.svg``
- Part 2: ``cesnna-knobox-enclosure2.svg``

Circuitry
---------

- Configure the potentiometers as voltage diviers:

  - Connect two ends to have 3.3v difference in voltage
  - Connect the pivot pins to A1 and A2 of the microcontroller

- Configure the switches as pull-up inputs:

  - Connect the middle pin of all switches to the ground (G/GND)
  - Parallel one side of the resistors (15K x5) to be 5V (5V pin)
  - Connect the other side of each resistor as shown in the diagram:

    - One (left/right) pin from each of three switches connects to A8, A9, A10, respectively
    - Pins (left & right) of the fourth switch connect to B14, B15.
    - Note: the lower right switch is a SPDT switch that springs to off when no
      pressure is applied. It could be used as flap control.

.. raw:: html

    <div align="center">
    <img src="https://raw.githubusercontent.com/Determinant/cessna-knobox/master/cessna-knobox-circuit.svg" width="70%">
    </div>


Firmware
--------

- (optional) To build the firmware from source, run ``./build_image.sh`` (make sure you
  have Rust/Cargo installed on your computer)
- Or you can use a pre-built image already available at ``./cessna-knobox.bin``
- Flash the image to the microcontroller using ST-link (you can either use the tool provided officially by ST, or do it in openocd)

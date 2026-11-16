# ShadowrunBAV

A Windows app for designing vehicles and drones from scratch by the rules of *Shadowrun, Sixth World: Double Clutch* ("Build Your Dream", pp. 147–157), and for modding them ("Making It Yours", pp. 120–144).

Pick a chassis, powertrain, and console, then add design options, design modifications, and quality factors. The app works out:

- the SR6 stat line (Hand, Accel, Spd Int, Top Spd, Body, Armor, Pilot, Sensor, Seat, Avail, Cost)
- the speed calculation, step by step
- the Build Points, line by line, with the quality factors' multipliers
- the price, Availability, and the Engineering + Logic tests to design and build it
- its mods: installed afterward (shop or do-it-yourself price, install test) or built into the design, with mod slots, hardpoints, and cargo space checked and the stat line updated
- weapons, typed in and fitted in weapon mounts
- the stat block laid out as ShadowrunEdit does (gear book or gamemaster), with Copy to paste it there
- a PDF vehicle sheet laid out like the book's entries

A design that breaks a rule still calculates, and the app says what to fix. Designs save as `.bav` files.

You need the book: the app holds only the names and numbers its calculations use, never the book's rules text. Where the book contradicts itself, the choices made are listed in [DESIGN.md](DESIGN.md) under "Rules decisions".


## Download

Installers are on the [Releases](../../releases) page. They aren't code-signed, so Windows SmartScreen shows "Windows protected your PC" the first time: choose **More info**, then **Run anyway**.

Once installed, ShadowrunBAV checks for updates by itself and shows an "Update to …" button when there is one; Check for updates on the toolbar checks right away. (Version 0.1.1 has no updater: install the next version by hand once.)

## Building from source

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

The app is open source under the [MIT License](LICENSE): use, change, and share it freely, keeping the copyright notice. Contributions are welcome; see CONTRIBUTING.md.

Shadowrun is a trademark of The Topps Company, Inc. This is an unofficial fan tool, not affiliated with Topps or Catalyst Game Labs.

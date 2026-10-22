# Tuxeren Control Center (TxeCtrl): Reverse Engineering, Driver Adaptation, and Hardware Control Development Process

This document records the technical journey of addressing the lack of hardware control on Monster laptops after migrating from Windows to Linux, the reverse engineering steps applied to the vendor software, the driver-level compatibility hurdles, and the from-scratch development of the **Tuxeren Control Center (`txectrl`)** project.
---

## 1. Background and Starting Point

When transitioning from Windows to Linux (Fedora), it became apparent that no official control center software was provided by the manufacturer for Monster laptops (target hardware: Monster Tulpar T7 V20.7 / Tongfang-Uniwill `GM7AR0R` chassis). The device's fan speeds, CPU power limits (PL1/PL2/PL4), keyboard backlighting, and the physical mode button on the chassis were unmanaged under Linux.

Initial research revealed that Monster laptops utilize Tongfang and Uniwill ODM (Original Design Manufacturer) chassis, which are also used by the German Linux PC vendor Tuxedo Computers. Theoretically, Tuxedo's open-source software was expected to work on this hardware as well.

However, practical testing encountered several roadblocks:
1. **Lack of Resources and Outdated Documentation:** Most online guides and instructions were obsolete.
2. **UI Sluggishness and Bloat:** When Tuxedo Control Center (TCC) was built and run directly from source, the interface proved quite sluggish and visually lagged behind modern standards.
3. **Empty Telemetry and Non-Functional Hardware:** Most critically, despite the interface launching, no hardware telemetry (fan RPMs, temperatures, power profiles) was displayed; the controls were completely ineffective.

### Root Cause: DMI / Vendor Identity Restriction
Upon inspecting the source code of the Tuxedo drivers (`tuxedo-drivers`), the following check was identified in `tuxedo_compatibility_check.c`:

```c
// Going forward we only run the drivers on in house tested devices
static const struct dmi_system_id tuxedo_dmi_string_match[] = {
    {
        .matches = {
            DMI_MATCH(DMI_SYS_VENDOR, "TUXEDO"),
        },
    },
    ...
};
```

Tuxedo enforced that the BIOS DMI table's `DMI_SYS_VENDOR` field must match `"TUXEDO"` so that the driver would only run on their internally validated devices. Even though Monster shares the exact same motherboard and chassis, it writes its own brand identity into the BIOS DMI tables:
- `DMI_SYS_VENDOR`: `"MONSTER"`
- `DMI_BOARD_NAME`: `"GM7AR0R"`
- `DMI_PRODUCT_NAME`: `"TULPAR T7 V20.7"`

Consequently, even if the driver appeared to load, it failed the compatibility check, did not instantiate hardware nodes, and provided no telemetry feed to the interface.

---

## 2. Reverse Engineering Process: Deciphering the Windows Control Center

Rather than relying on third-party drivers to solve the issue, the decision was made to decode the hardware's native protocol directly. The Windows version corresponding to the device, `ControlCenter_5.8.49.19_Monster.exe`, was downloaded from Monster's official support page, initiating the reverse engineering workflow.

### 2.1 Binary Parsing and Architectural Layers
The installer was identified as an **Inno Setup 6.3.0** container and unpacked. Crucially, the extracted files included the 158 MB `GamingCenter3_Cross.pdb` symbol file, which allowed all internal functions, structs, and symbols to be decoded directly.

The analysis revealed the three-tier architecture of the Windows software:
- **User Interface:** UWP C++/CX-based `GamingCenter3_Cross.dll` and `SystrayComponent.exe` managing the system tray.
- **Service and Communication:** A .NET Framework 4.7.2 service (`GCUService.exe`) running in the background as `NT AUTHORITY\SYSTEM` hosting a local MQTT broker (`127.0.0.1:1883`).
- **Hardware Bridge:** The KMDF kernel driver `UWACPIDriver.sys` called through `ACPIDriverDll.dll`, targeting the ACPI-level device node: `ACPI\INOU0000`.

---

## 3. IOCTL Dispatch Table and Resolving the ACPI Bridge

To understand how the Windows service accessed the underlying hardware, the `ACPIDriverDll.dll` and `UWACPIDriver.sys` binaries were disassembled. It was determined that the driver exposed the `\\.\ACPIDriver` symbolic link to handle IOCTL requests adhering to the standard Windows `CTL_CODE` specification (`DeviceType = 0x9C40`):

$$\text{IOCTL} = (\text{DeviceType} \ll 16) \mid (\text{RequiredAccess} \ll 14) \mid (\text{Function} \ll 2) \mid \text{TransferType}$$

| Function Name | Export No | IOCTL Code | Input / Output | Description |
| :--- | :--- | :--- | :--- | :--- |
| `ReadCMOS` | 1 | `0x9C40A480` | `u32` offset $\rightarrow$ `u8` value | Read CMOS RTC |
| `WriteCMOS` | 14 | `0x9C40A484` | `[u32; 2]` `[reg, val]` | Write CMOS RTC |
| `ReadEC` | 2 | `0x9C40A488` | `u32` offset $\rightarrow$ `u8` value | Read 1 byte from EC RAM |
| `WriteEC` | 15 | `0x9C40A48C` | `[u32; 2]` `[reg, val]` | Write 1 byte to EC RAM |
| `ReadIO` | 3 | `0x9C40A4C0` | `u16` port $\rightarrow$ `u8` value | x86 I/O port read (`inb`) |
| `WriteIO` | 16 | `0x9C40A4C4` | `[u16; 1], [u8; 1]` | x86 I/O port write (`outb`) |
| `SMAPCTable` | 7 | `0x9C40A500` | 128 bytes input/output | Smart APC power synchronization |

### Critical Discovery: Direct ACPI Method Invocation
Upon disassembling `UWACPIDriver.sys`, it was found that the `ReadEC` function executed the `\_SB.INOU.ECRR` method with an ACPI evaluation buffer, and `WriteEC` executed the `\_SB.INOU.ECRW` method.

This finding eliminated the necessity of developing a proprietary Linux kernel driver simply to communicate with the hardware; the exact same ACPI methods could be executed directly via the standard `acpi_call` kernel module (`/proc/acpi/call`).

---

## 4. Extracting the Embedded Controller (EC) Register Map

By decompiling `GCUService.exe`, the classes `Define.ECSpec`, `Define.RamFan1p5_ECSpec`, and `Define.RamFan2_ECSpec` were analyzed. This yielded the comprehensive memory map of the Embedded Controller on the motherboard:

### 4.1 Fan Control and Telemetry Distinction
- `0x075B` & `0x075C`: CPU and GPU instantaneous fan duty cycles (`0%` - `100%` PWM).
- `0x0751`: **Main Fan Mode Controller:**
  - `0x00`: Automatic BIOS control
  - `0x10`: Turbo mode
  - `0x40`: CoolerBoost (100% full speed lock)
  - `0x80`: Custom fan curve (RamFan active)

> [!NOTE]
> **Semantic Ambiguity at Address `0x046B` and Fan Telemetry:**  
> In the Windows PDB debug symbols (`Define.ECSpec`), `0x0464` is labeled CPU Fan RPM and `0x046B` GPU Fan RPM. However, live telemetry analysis on real Tongfang/Uniwill (`GM7AR0R`) hardware demonstrated that address `0x046A` corresponds to **Live PL1**, and address `0x046B` corresponds to **Live PL2** power limits (Watts). Because these addresses are allocated to dynamic power limits on the live system, fan speed telemetry in our Rust architecture was designed to read from the standard kernel `/sys/class/hwmon` interface (`fan1_input`, `fan2_input`) instead of directly sampling EC RAM bytes.

### 4.2 RamFan 11-Point Fan Curve Tables (`0x0F00` - `0x0F5F`)
The system maintains custom fan curve profiles in the upper EC RAM area:
- `0x0F00`: CPU temperature up-threshold triggers (`UpT`, 11 bytes)
- `0x0F10`: CPU temperature down-threshold triggers (`DownT` hysteresis, 11 bytes)
- `0x0F20`: CPU fan PWM duty percentages (11 bytes)
- `0x0F30` - `0x0F50`: Corresponding 11-byte tables for the GPU
- `0x0F5F`: Trigger byte that commits and applies the table to the EC

### 4.3 CPU and GPU Power Limits
- `0x07AB`: Active power mode index (1: Gaming, 2: Office, 3: Battery Saver, 4: Turbo).
- `0x0783` (PL1), `0x0784` (PL2), `0x0785` (PL4): CPU static BIOS profile limits (Watts).
- `0x0738` (Tau): PL1 time window duration (seconds).
- `0x0786` (TCC Offset): CPU thermal throttling offset (TjMax - offset).
- `0x0743` - `0x0746`: GPU Configurable TGP and Dynamic Boost power targets.

### 4.4 Keyboard Backlight
- `0x0769`, `0x076A`, `0x076B`: Red, Green, Blue brightness levels (`0..50`).
- `0x078C`: Illumination power switch and brightness state step.
- `0x0767`: Latching byte to commit new values to hardware (Bit `0x20`).

---

## 5. Initial Live Verification: `acpi_call` and Python Prototype

To verify the extracted register addresses, a minimal Python test script was developed under the `py_deneme` prototype directory, interacting with the Linux `/proc/acpi/call` bridge.

Script execution flow:
```python
# 1. Write to color registers:
write_ec(0x0769, red)    # Red PWM (0..50)
write_ec(0x076A, green)  # Green PWM (0..50)
write_ec(0x076B, blue)   # Blue PWM (0..50)

# 2. Set brightness level:
write_ec(0x078C, brightness)

# 3. Trigger latch bit:
write_ec(0x0767, 0x20)

# 4. Re-arm strobe so the hardware updates color immediately:
write_ec(0x078C, 0x00)
time.sleep(0.04)
write_ec(0x078C, 0x01)
```

The keyboard hardware responded instantly to this test, changing colors as expected. In addition, photopic calibration coefficients reflecting human visual perception were applied to balance the color channels, enabling accurate rendition of intermediate hues (orange, cyan, purple).

### Assumptions Disproved During Testing:
1. **USB HID Fallacy:** Despite references to ITE IT829x USB HID libraries in the Windows code, the device was absent from `lsusb` output; on the Tulpar T7 V20.7, the keyboard was proven to be a **Single-Zone PWM** LED array wired directly to the EC.
2. **Fedora Kernel Incompatibility Myth:** The failure of off-the-shelf Linux utilities was not caused by a lack of kernel hardware support, but because the `ec_sys` debugfs module is intentionally omitted from Fedora kernel builds for security reasons. The ACPI node at `/sys/bus/acpi/devices/INOU0000:00/` was fully functional.

---

## 6. Strategic Decision: Custom Module vs. Inheriting Tuxedo Drivers

With keyboard and EC access verified, two technical paths emerged:
1. **Option A:** Write an entire EC management and ACPI interaction layer from scratch as a custom Rust-based kernel driver or userspace daemon.
2. **Option B:** Fork and patch Tuxedo's open-source `tuxedo-drivers` kernel module—benefiting from 5+ years of battle-tested edge cases, ACPI WMI event handling, and suspend/resume lifecycle management on Uniwill and Tongfang chassis.

### Decision and Implementation
Rather than reinventing the wheel, we chose to leverage Tuxedo's five years of field-tested engineering. The DMI filtering mechanism in the driver source was patched by inserting Monster hardware signatures into `tuxedo_compatibility_check.c` and `clevo_leds.h`:

```c
// driver/tuxedo-drivers-monster-4.25.0/src/tuxedo_compatibility_check/tuxedo_compatibility_check.c
static const struct dmi_system_id tuxedo_dmi_string_match[] = {
    { .matches = { DMI_MATCH(DMI_SYS_VENDOR, "TUXEDO"), }, },
    { .matches = { DMI_MATCH(DMI_SYS_VENDOR, "MONSTER"), }, },
    { .matches = { DMI_BOARD_NAME, "GM7AR0R"), }, },
    { .matches = { DMI_BOARD_NAME, "TULPAR T7 V20.7"), }, },
    { .matches = { DMI_PRODUCT_SKU, "T7V207GM7AR0R"), }, },
    ...
};
```

This patch enabled `tuxedo_keyboard`, `uniwill_wmi`, and `tuxedo_io` kernel modules to initialize standard Linux `sysfs` LED and WMI interfaces (`/sys/devices/platform/tuxedo_keyboard/`) flawlessly on the Monster Tulpar device.

---

## 7. Deciphering the Chassis Mode Button and Dynamic Telemetry Node

During live operational testing, the most challenging obstacle was the physical performance mode button located on the chassis and its integrated indicator LED:

### 7.1 Autonomous Hardware State Machine
Static Windows code suggested that pressing the button would raise an ACPI scancode `0x00B0` to the operating system. Under Linux, however, neither `acpi_listen` nor `/dev/input/event*` monitors captured any scancode. It was discovered that the button operates as an autonomous hardware state machine handled directly by the EC firmware, transitioning internal states without waiting for OS intervention.

### 7.2 The Non-Extinguishing LED Problem and Address `0x0751`
Chassis button presses were tracked to mutate state directly through `0x0751` (`ADDR_MAFAN_CONTROL_BYTE`), rather than `0x07AB`.

Existing open-source implementations wrote `0x00` (Automatic BIOS) to this address when switching to silent mode. On this specific chassis, while writing `0x00` restored fan control to automatic mode, it failed to turn off the button LED, leaving it illuminated at half brightness (`0x20`).

Empirical telemetry testing demonstrated that completely extinguishing the LED requires explicitly writing **`0xA0`** to this address:

| Operating Mode | Button LED Status | `0x0751` Value | Live PL1 (`0x046A`) | Live PL2 (`0x046B`) | Characteristics |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Office / Silent** | **Fully Extinguished** | `0xA0` | `45 W` (`0x2D`) | `45 W` (`0x2D`) | Acoustic priority, LED off |
| **Gaming / Balanced**| **Half Brightness (Single LED)** | `0x20` | `75 W` (`0x4B`) | `75 W` (`0x4B`) | Balanced thermal envelope, single LED |
| **Turbo / Beast** | **Full Brightness (Dual LED)** | `0x30` | `90 W` (`0x5A`) | `90 W` (`0x5A`) | Maximum TDP, dual LED on |

### 7.3 Dynamic Limits (`0x046A` / `0x046B`) and Hwmon Fan Polling
When the physical mode button is pressed, the EC copies BIOS default power profiles directly into the live registers `0x046A` (PL1) and `0x046B` (PL2).

Contrary to the static symbol definition indicating `0x046B` as GPU RPM, on live hardware `0x046A` and `0x046B` represent the 8-bit dynamic CPU power ceilings (45W, 75W, 90W). Consequently, our Rust core (`performance.rs`) writes directly to these registers during mode transitions; fan RPM readings are polled via the kernel's `/sys/class/hwmon` interface (`find_hwmon_fans`) to prevent concurrent port collisions on the EC.

---

## 8. Developing Custom Software: Tuxeren Branding and Modern Linux Architecture

The scope was expanded from a single Monster model to support any laptop sharing the same ODM architecture. At this stage, the project was branded as **Tuxeren Control Center (`txectrl`)**.

```
[Chassis Physical Button / UI Interaction]
                 │
                 ▼
     [Rust Hardware Controller]
                 │
    ┌─────────────┴─────────────┐
    ▼                           ▼
[EC RAM & Driver Sync]       [Linux Kernel Integration]
- 0x0751 (Fan & Mode LED)    - Intel RAPL powercap (/sys/class/powercap/intel-rapl)
- 0x046A / 0x046B (PL1/PL2)  - CPU cpufreq EPP (power / balance / performance)
- sysfs /dev/tuxedo_io       - D-Bus power-profiles-daemon synchronization
```

### 8.1 Architectural Components
1. **Rust Core and Hardware Manager:**
   - A lightweight hardware poller sampling `0x0751` every 200 ms with CPU consumption below 0.01%.
   - A synchronization engine enforcing microwatt-level bounds via Linux `intel-rapl` (`powercap`) and updating `cpufreq/energy_performance_preference` policies when modes change.
2. **Tauri v2 Native UI:**
   - Replacing the heavyweight architecture of legacy Tuxedo software with a lightweight, responsive Tauri v2 frontend backed by the Rust engine.
   - 11 color presets, a 2D HSV color picker, real-time telemetry dashboard, and full English/Turkish bilingual support.

### 8.2 Kernel Module Lifecycle and Compilation Strategy
The lifecycle roles for the two kernel drivers utilized in the project are distinctly separated:

- **Early Prototyping and Raw EC Exploration (`acpi_call`):**  
  Used during initial hardware discovery and direct EC RAM validation tests via `/proc/acpi/call`. If kernel updates in Fedora lead to module absence or `vermagic` mismatch, `acpi_call.ko` is rebuilt on the fly against local kernel headers (`/lib/modules/$(uname -r)/build`).

- **Production Deployment (`tuxedo-drivers` & DKMS):**  
  The core drivers (`tuxedo_keyboard`, `uniwill_wmi`, `tuxedo_io`) are enrolled under **DKMS (Dynamic Kernel Module Support)** at `/usr/src/tuxedo-drivers-monster-4.25.0` via `deploy_driver.sh` and single-click `pkexec` elevation in the GUI. During kernel upgrades, DKMS automatically recompiles the modules for the incoming kernel. The same deployment script installs `99-tuxeren-control-center.rules` to configure unprivileged user access to `/sys/devices/platform/tuxedo_keyboard` and `/dev/tuxedo_io` without requiring root permissions.

---

## 9. Conclusion and Outcomes

Through this engineering effort:
- The closed-source Windows control center's IOCTL and EC protocols were fully reverse engineered.
- The DMI compatibility restriction preventing Tuxedo drivers from loading on Monster hardware was patched.
- A modern, lightweight, and responsive control center ([Tuxeren Control Center](README.md)) running in full synchronization with the Linux kernel was successfully engineered and deployed.

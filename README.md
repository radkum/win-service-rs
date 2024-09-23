# windows-service-rs

The simple windows service written in Rust.

### Windows service description
The system windows supports services. Service is a process that typically work from user login to logout. 

To learn more details see: https://learn.microsoft.com/en-us/windows/win32/services/services

### Project description
This sample is very simple and has one purpose, to increase counter by 1 per second, and log out counter 
with OutputDebugStringA. You can see these output with DbgView (don't forget turn on "Capture global Win32")

There is one interesting feature in this service. It supports one User Control code that set protection level 
to 0 (SERVICE_LAUNCH_PROTECTED_NONE ). In few words it unprotect itself. 

It's necessary feature to test PPL services.

### Getting Started
There are no special steps to make this work. You can simply run build command `cargo b`

The signed `win-service.exe` is produced into `target/debug` directory

### How to run
You have several options:
1. Use SCM (service control manager) to create and start service.
2. Create own application to manage this service

Few SCM commands. Lets assume you service name is "sample":
- `sc create sample binPath=<path_to_service_exe>`
- `sc start sample`
- `sc qc sample`
- `sc stop sample`
- `sc delete sample`

### Links:
- https://learn.microsoft.com/en-us/windows/win32/services/services
- https://github.com/mullvad/windows-service-rs
- https://learn.microsoft.com/en-us/windows/win32/services/service-control-manager
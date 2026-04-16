# EDR

usage:

- first disable in bios "Secure Boot"
- you need signtool.exe (from WDK)
- dbgview downloaded

run powershell as admin to create certificate (only once):
$cert = New-SelfSignedCertificate -Subject "CN=RustTest" -Type CodeSigningCert -CertStoreLocation "Cert:\LocalMachine\My"
Export-Certificate -Cert $cert -FilePath "C:\RustTest.cer"
Import-Certificate -FilePath "C:\RustTest.cer" -CertStoreLocation "Cert:\LocalMachine\Root"

create registry keys for the driver:
$RegPath = "HKLM:\SYSTEM\CurrentControlSet\Services\rustdriver"

New-Item -Path "$RegPath\Instances" -Force | Out-Null
New-ItemProperty -Path "$RegPath\Instances" -Name "DefaultInstance" -Value "RustDriverInstance" -PropertyType String -Force | Out-Null

New-Item -Path "$RegPath\Instances\RustDriverInstance" -Force | Out-Null
New-ItemProperty -Path "$RegPath\Instances\RustDriverInstance" -Name "Altitude" -Value "370000" -PropertyType String -Force | Out-Null
New-ItemProperty -Path "$RegPath\Instances\RustDriverInstance" -Name "Flags" -Value 0 -PropertyType DWord -Force | Out-Null

then sign the driver:
& "C:\Users\Administrator\Desktop\signtool.exe" sign /v /sm /fd SHA256 /n "RustTest" /t http://timestamp.digicert.com C:\Users\Administrator\Desktop\rust_driver.sys


sc.exe create rustdriver type= kernel binPath= "C:\Users\Administrator\Desktop\rust_driver.sys"
sc.exe start rustdriver
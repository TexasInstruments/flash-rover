@echo off

set ECLIPSEC=C:\ti\ccs\ccsv9_0_1\ccs\eclipse\eclipsec.exe

set PROJECTSPEC_CC13X0=C:\ti\git\flash-rover\src\fw\gcc\cc13x0-cc26x0\flash_rover_fw_cc13x0.projectspec
set PROJECTSPEC_CC26X0=C:\ti\git\flash-rover\src\fw\gcc\cc13x0-cc26x0\flash_rover_fw_cc26x0.projectspec
set PROJECTSPEC_CC26X0R2=C:\ti\git\flash-rover\src\fw\gcc\cc13x0-cc26x0\flash_rover_fw_cc26x0r2.projectspec
set PROJECTSPEC_CC13X2_CC26X2=C:\ti\git\flash-rover\src\fw\gcc\cc13x2-cc26x2\flash_rover_fw_cc13x2_cc26x2.projectspec

set WORKSPACE=C:\ti\git\flash-rover\src\fw\workspace
set FIRMWARE_CC13X0=%WORKSPACE%\flash_rover_fw_cc13x0_gcc\Firmware\cc13x0.bin
set FIRMWARE_CC26X0=%WORKSPACE%\flash_rover_fw_cc26x0_gcc\Firmware\cc26x0.bin
set FIRMWARE_CC26X0R2=%WORKSPACE%\flash_rover_fw_cc26x0r2_gcc\Firmware\cc26x0r2.bin
set FIRMWARE_CC13X2_CC26X2=%WORKSPACE%\flash_rover_fw_cc13x2_cc26x2_gcc\Firmware\cc13x2_cc26x2.bin

"%ECLIPSEC%" -noSplash -data "%WORKSPACE%" -application com.ti.ccstudio.apps.projectImport -ccs.overwrite -ccs.autoImportReferencedProjects true -ccs.location "%PROJECTSPEC_CC13X0%" -ccs.location "%PROJECTSPEC_CC26X0%" -ccs.location "%PROJECTSPEC_CC26X0R2%" -ccs.location "%PROJECTSPEC_CC13X2_CC26X2%"

"%ECLIPSEC%" -noSplash -data "%WORKSPACE%" -application com.ti.ccstudio.apps.projectBuild -ccs.projects flash_rover_fw_cc13x0_gcc flash_rover_fw_cc26x0_gcc flash_rover_fw_cc26x0r2_gcc flash_rover_fw_cc13x2_cc26x2_gcc -ccs.configuration Firmware -ccs.buildType full

copy "%FIRMWARE_CC13X0%" "..\cli\dss\fw"
copy "%FIRMWARE_CC26X0%" "..\cli\dss\fw"
copy "%FIRMWARE_CC26X0R2%" "..\cli\dss\fw"
copy "%FIRMWARE_CC13X2_CC26X2%" "..\cli\dss\fw"

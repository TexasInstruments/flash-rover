@echo off

set CCS_ROOT=C:\ti\ccs\ccsv9_0_1\ccs
set ECLIPSEC=%CCS_ROOT%\eclipse\eclipsec.exe


set SCRIPT_ROOT=%~dp0
set PROJECT_ROOT=%SCRIPT_ROOT%\..
set FW_DEST=%PROJECT_ROOT%\cli\dss\fw

set PROJECTSPEC_CC13X0=%PROJECT_ROOT%\fw\gcc\cc13x0-cc26x0\flash_rover_fw_cc13x0.projectspec
set PROJECTSPEC_CC26X0=%PROJECT_ROOT%\fw\gcc\cc13x0-cc26x0\flash_rover_fw_cc26x0.projectspec
set PROJECTSPEC_CC26X0R2=%PROJECT_ROOT%\fw\gcc\cc13x0-cc26x0\flash_rover_fw_cc26x0r2.projectspec
set PROJECTSPEC_CC13X2_CC26X2=%PROJECT_ROOT%\fw\gcc\cc13x2-cc26x2\flash_rover_fw_cc13x2_cc26x2.projectspec

set WORKSPACE=%PROJECT_ROOT%\fw\workspace
set FW_CC13X0=%WORKSPACE%\flash_rover_fw_cc13x0_gcc\Firmware\cc13x0.bin
set FW_CC26X0=%WORKSPACE%\flash_rover_fw_cc26x0_gcc\Firmware\cc26x0.bin
set FW_CC26X0R2=%WORKSPACE%\flash_rover_fw_cc26x0r2_gcc\Firmware\cc26x0r2.bin
set FW_CC13X2_CC26X2=%WORKSPACE%\flash_rover_fw_cc13x2_cc26x2_gcc\Firmware\cc13x2_cc26x2.bin

"%ECLIPSEC%" -noSplash -data "%WORKSPACE%" ^
 -application com.ti.ccstudio.apps.projectImport -ccs.overwrite ^
  -ccs.autoImportReferencedProjects true ^
  -ccs.location "%PROJECTSPEC_CC13X0%" ^
  -ccs.location "%PROJECTSPEC_CC26X0%" ^
  -ccs.location "%PROJECTSPEC_CC26X0R2%" ^
  -ccs.location "%PROJECTSPEC_CC13X2_CC26X2%"

"%ECLIPSEC%" -noSplash -data "%WORKSPACE%" -application com.ti.ccstudio.apps.projectBuild ^
 -ccs.projects flash_rover_fw_cc13x0_gcc ^
 flash_rover_fw_cc26x0_gcc ^
 flash_rover_fw_cc26x0r2_gcc ^
 flash_rover_fw_cc13x2_cc26x2_gcc ^
 -ccs.configuration Firmware -ccs.buildType full

copy "%FW_CC13X0%" "%FW_DEST%"
copy "%FW_CC26X0%" "%FW_DEST%"
copy "%FW_CC26X0R2%" "%FW_DEST%"
copy "%FW_CC13X2_CC26X2%" "%FW_DEST%"

@ECHO OFF
rem Copyright (c) 2020 , Texas Instruments.
rem Licensed under the BSD-3-Clause license
rem (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
rem notice may not be copied, modified, or distributed except according to those terms.

SETLOCAL enableextensions

SET /A ERRNO=0

SET CURR_DIR=%~dp0

rem Use CCS_ROOT if already set, otherwise calculate it from this script's location
rem Default assumes placement at <CCS_ROOT>\utils\flash-rover\flash-rover.bat
IF NOT DEFINED CCS_ROOT (
    SET CCS_ROOT=%CURR_DIR%..\..
)

SET LIBJVM_PATH=%CCS_ROOT%\eclipse\jre\bin\server

IF NOT EXIST "%LIBJVM_PATH%\jvm.dll" (
    ECHO ERROR: Could not find jvm.dll 1>&2
    ECHO CCS_ROOT is set to: %CCS_ROOT% 1>&2
    ECHO If CCS is not installed in the default location, set the CCS_ROOT variable: 1>&2
    ECHO   set CCS_ROOT=C:\ti\ccs2020\ccs 1>&2
    SET /A ERRNO=1
    GOTO :EXIT
)

rem Setup environment for JRE
SET PATH=%LIBJVM_PATH%;%PATH%

rem Call flash-rover executable
"%CURR_DIR%\ti-xflash.exe" %*
SET /A ERRNO=%ERRORLEVEL%

:EXIT
ECHO ON
@EXIT /B %ERRNO%

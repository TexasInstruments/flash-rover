@ECHO OFF
SETLOCAL enableextensions

SET /A ERRNO=0

SET CURR_DIR=%~dp0
SET LIBJVM_PATH=%CURR_DIR%..\..\eclipse\jre\bin\server

IF NOT EXIST "%LIBJVM_PATH%\jvm.dll" (
    ECHO libjvm path configured in flash-rover.bat is wrong, please verify LIBJVM_PATH is correct before continuing 1>&2
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

@echo off
setlocal
set "IMPECCABLE_HOME=%~dp0..\.impeccable\runtime"
cd /d "%~dp0.."
call ".agents\skills\impeccable\scripts\impeccable.cmd" %*
exit /b %errorlevel%

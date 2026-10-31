@echo off
cd /d "%~dp0"
python independent-engine\serve.py --lan
pause

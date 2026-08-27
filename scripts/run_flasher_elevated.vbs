Set objShell = CreateObject("Shell.Application")
Dim exePath, args
exePath = "C:\Users\ROG G532 LV\.gemini\antigravity\scratch\catalyst-os\target\RawWriter.exe"
args = "--yes-flash-disk1-sandisk"
objShell.ShellExecute exePath, args, "", "runas", 1

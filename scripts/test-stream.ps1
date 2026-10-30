$proc = Start-Process target\release\PrysmHost.exe -PassThru -RedirectStandardError "target\host_err.log" -RedirectStandardOutput "target\host_out.log"
Start-Sleep -Seconds 2
curl.exe -v -o target\test_frame.jpg "http://127.0.0.1:19533/frame"
Start-Sleep -Seconds 1
(Get-Item target\test_frame.jpg).Length
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Get-Content "target\host_err.log" -ErrorAction SilentlyContinue
Get-Content "target\host_out.log" -ErrorAction SilentlyContinue

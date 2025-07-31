# Configure Systemd
* Check paths in `start.sh` and `mygriddash.service`
* Copy `mygriddash.service` to `/lib/systemd/system/`
* Run `sudo systemctl enable mygriddash.service`
* Run `sudo systemctl start mygriddash.service`
* Check status by running `sudo systemctl status mygriddash.service`

Output should be something like:
```
● mygriddash.service - Dash for MyGrid
     Loaded: loaded (/lib/systemd/system/mygriddash.service; enabled; preset: enabled)
     Active: active (running) since Thu 2025-07-31 10:22:09 CEST; 6s ago
   Main PID: 137204 (bash)
      Tasks: 8 (limit: 9573)
        CPU: 360ms
     CGroup: /system.slice/mygriddash.service
             ├─137204 /bin/bash /home/petste/MyGridDash/start.sh
             └─137205 /home/petste/MyGridDash/mygrid_dash --config=/home/petste/MyGridDash/config/config.toml

Jul 31 10:22:09 mygrid systemd[1]: Started mygriddash.service - Dash for MyGrid.
```

If the application for some reason prints anything to stdout/stderr, such in case of a panic,
the log for that can be found by using `journalctl -u mygriddash.service`.
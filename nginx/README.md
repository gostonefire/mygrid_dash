# Configure nginx
## Install nginx
* sudo apt update
* sudo apt install -y nginx
* sudo systemctl enable --now nginx

## Production
### Create the server block for dash.gridfire.org
* sudo nano /etc/nginx/sites-available/dash.gridfire.org.conf
```text
server {
    listen 80;
    listen [::]:80;
    server_name dash.gridfire.org;

    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name dash.gridfire.org;

    # <-- CHANGE THESE PATHS TO YOUR REAL CERT FILE LOCATIONS
    ssl_certificate     /etc/letsencrypt/live/gridfire.org-0001/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/gridfire.org-0001/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # If you do uploads, you can raise this
    client_max_body_size 50m;

    # Standard reverse-proxy headers
    proxy_set_header Host              $host;
    proxy_set_header X-Real-IP         $remote_addr;
    proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    # Special case: anything starting with /deploy goes to gridfire:8086
    # This preserves /deploy in the forwarded URI, so:
    #   /deploy          -> http://mygrid.gridfire.org:8086/deploy
    #   /deploy/foo      -> http://mygrid.gridfire.org:8086/deploy/foo
    location ^~ /deploy {
        proxy_http_version 1.1;
        proxy_pass http://mygrid.gridfire.org:8086;
    }

    # Default: everything else goes to mygrid:8085
    location / {
        proxy_http_version 1.1;
        proxy_pass http://mygrid.gridfire.org:8085;
    }
}
```
### Enable the site and reload Nginx
* sudo ln -s /etc/nginx/sites-available/dash.gridfire.org.conf /etc/nginx/sites-enabled/
* sudo rm -f /etc/nginx/sites-enabled/default
* sudo nginx -t
* sudo systemctl reload nginx

## Test
### Create the server block for test.gridfire.org
* sudo nano /etc/nginx/sites-available/test.gridfire.org.conf
```text
server {
    listen 80;
    listen [::]:80;
    server_name test.gridfire.org;

    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name test.gridfire.org;

    # <-- CHANGE THESE PATHS TO YOUR REAL CERT FILE LOCATIONS
    ssl_certificate     /etc/letsencrypt/live/gridfire.org-0001/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/gridfire.org-0001/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_session_cache shared:SSL:10m;
    ssl_session_timeout 10m;

    # If you do uploads, you can raise this
    client_max_body_size 50m;

    # Standard reverse-proxy headers
    proxy_set_header Host              $host;
    proxy_set_header X-Real-IP         $remote_addr;
    proxy_set_header X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;

    # Special case: anything starting with /deploy goes to gridfire:8086
    # This preserves /deploy in the forwarded URI, so:
    #   /deploy          -> http://hobbylap.gridfire.org:8086/deploy
    #   /deploy/foo      -> http://hobbylap.gridfire.org:8086/deploy/foo
    location ^~ /deploy {
        proxy_http_version 1.1;
        proxy_pass http://hobbylap.gridfire.org:8086;
    }

    # Default: everything else goes to hobbylap:8085
    location / {
        proxy_http_version 1.1;
        proxy_pass http://hobbylap.gridfire.org:8085;
    }
}
```
### Enable the site and reload Nginx
* sudo ln -s /etc/nginx/sites-available/test.gridfire.org.conf /etc/nginx/sites-enabled/
* sudo nginx -t
* sudo systemctl reload nginx

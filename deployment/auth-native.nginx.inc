# Include ONLY inside the existing auth.solara-inc.in HTTPS server, after review.
# Existing SSO/login routes continue to point to the existing account service.
location ^~ /native/ {
    client_max_body_size 64k;
    # limit_req zone=solara_launcher_auth burst=10 nodelay;
    # Disable query-string logs here: authorization codes/state are not operational telemetry.
    access_log off;
    include fastcgi_params;
    fastcgi_param SCRIPT_FILENAME /var/www/solara-launcher-private/public/index.php;
    fastcgi_param HTTP_AUTHORIZATION $http_authorization;
    fastcgi_pass unix:/run/php/php8.4-fpm.sock;
}

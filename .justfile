set dotenv-filename := ".env.just"
project_name := "meshhat-controller"
remote_user:= "ubuntu"
remote_host:= "10.0.0.7"

remote-snap:
    #!/usr/bin/env bash
    set -euo pipefail

    if [[ -z "{{remote_user}}" || -z "{{remote_host}}" ]]; then
        echo "Error: remote_user and remote_host must be set in .env.just"
        exit 1
    fi

    tar -czf /tmp/snap.tar.gz snap/ {{project_name}}/
    scp /tmp/snap.tar.gz {{remote_user}}@{{remote_host}}:/tmp/snap.tar.gz
    ssh {{remote_user}}@{{remote_host}} "rm -rf /tmp/{{project_name}} && mkdir -p /tmp/{{project_name}} && tar -xzf /tmp/snap.tar.gz -C /tmp/{{project_name}} && cd /tmp/{{project_name}} && snapcraft"
    scp {{remote_user}}@{{remote_host}}:/tmp/{{project_name}}/*.snap .
    rm /tmp/snap.tar.gz
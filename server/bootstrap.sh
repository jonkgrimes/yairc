#!/usr/bin/env bash

apt-get update

# wget https://github.com/inspircd/inspircd/archive/v2.0.25.tar.gz
wget "https://github.com/inspircd/inspircd/releases/download/v3.9.0/inspircd_3.9.0.ubuntu18.04.2_amd64.deb"

apt-get install "./inspircd_3.9.0.ubuntu18.04.2_amd64.deb" -y


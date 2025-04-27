FROM debian:bookworm

RUN apt-get update && apt-get upgrade -y && apt-get install -y build-essential libargon2-1 wget

# Download InspIRCd
RUN wget "https://github.com/inspircd/inspircd/releases/download/v4.7.0/inspircd_4.7.0.deb12u2_amd64.deb"

# Install InspIRCd
RUN apt-get install --upgrade -y ./inspircd_4.7.0.deb12u2_amd64.deb

# Copy configuration
COPY inspircd.conf /etc/inspircd/inspircd.conf

# Expose the IRC port
EXPOSE 6697

# Start InspIRCd service
CMD ["inspircd", "--runasroot", "--nofork"]

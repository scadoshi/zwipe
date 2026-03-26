#!/usr/bin/env bash
# pi-setup.sh — run once on the Pi to configure zerver
# Usage: bash pi-setup.sh
set -e

ZWIPE_DB_USER="zwipe"
ZWIPE_DB_NAME="zwipe"
INSTALL_DIR="/home/scottyfermo/zwipe"

echo "==> Creating install directory"
mkdir -p "$INSTALL_DIR"
mkdir -p "$INSTALL_DIR/migrations"

echo "==> Checking PostgreSQL"
if ! command -v psql &>/dev/null; then
  echo "Installing PostgreSQL..."
  sudo apt update && sudo apt install -y postgresql postgresql-contrib
fi
sudo systemctl enable postgresql
sudo systemctl start postgresql

echo "==> Creating DB user and database (skips if already exists)"
sudo -u postgres psql -tc "SELECT 1 FROM pg_roles WHERE rolname='$ZWIPE_DB_USER'" | grep -q 1 || \
  sudo -u postgres psql -c "CREATE USER $ZWIPE_DB_USER WITH PASSWORD '$ZWIPE_DB_PASS';"

sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname='$ZWIPE_DB_NAME'" | grep -q 1 || \
  sudo -u postgres psql -c "CREATE DATABASE $ZWIPE_DB_NAME OWNER $ZWIPE_DB_USER;"

echo "==> Running migrations"
cd "$INSTALL_DIR"
for f in migrations/*.sql; do
  echo "  Applying $f"
  PGPASSWORD="$ZWIPE_DB_PASS" psql -U "$ZWIPE_DB_USER" -d "$ZWIPE_DB_NAME" -f "$f" 2>/dev/null || true
done

echo "==> Writing .env"
cat > "$INSTALL_DIR/.env" <<ENV
JWT_SECRET=$ZWIPE_JWT_SECRET
DATABASE_URL=postgres://$ZWIPE_DB_USER:$ZWIPE_DB_PASS@localhost/$ZWIPE_DB_NAME
BIND_ADDRESS=0.0.0.0:3000
ALLOWED_ORIGINS=https://zwipe.net
RUST_LOG=info
RUST_BACKTRACE=0
ENV

echo "==> Installing zerver systemd service"
sudo tee /etc/systemd/system/zerver.service > /dev/null <<SERVICE
[Unit]
Description=zerver - zwipe backend
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=scottyfermo
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/.env
ExecStart=$INSTALL_DIR/zerver
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
SERVICE

echo "==> Installing zervice cron (nightly 4am)"
(crontab -l 2>/dev/null | grep -v zervice; echo "0 4 * * * $INSTALL_DIR/zervice >> $INSTALL_DIR/zervice.log 2>&1") | crontab -

sudo systemctl daemon-reload
sudo systemctl enable zerver

echo ""
echo "Setup complete. Start zerver with:"
echo "  sudo systemctl start zerver"
echo "  sudo systemctl status zerver"
echo ""
echo "NOTE: Before starting, copy zerver + zervice binaries and migrations/ to $INSTALL_DIR"

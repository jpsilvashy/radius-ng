#!/bin/bash
# test_radius.sh - Simple RADIUS testing script
# Usage: ./test_radius.sh [options]
#
# Options:
#   -h, --host HOST       RADIUS server hostname/IP (default: localhost)
#   -p, --port PORT       RADIUS authentication port (default: 1812)
#   -s, --secret SECRET   Shared secret (default: testing123)
#   -u, --user USERNAME   Username to test (default: testuser)
#   -w, --pass PASSWORD   Password to test (default: password)
#   -t, --type TYPE       Test type: auth, acct, or both (default: auth)
#   -c, --count COUNT     Number of requests to send (default: 1)
#   -d, --delay DELAY     Delay between requests in seconds (default: 0)
#   --help                Show this help message

set -e

# Default values
HOST="localhost"
PORT="1812"
SECRET="testing123"
USERNAME="testuser"
PASSWORD="password"
TYPE="auth"
COUNT=1
DELAY=0

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    -h|--host)
      HOST="$2"
      shift 2
      ;;
    -p|--port)
      PORT="$2"
      shift 2
      ;;
    -s|--secret)
      SECRET="$2"
      shift 2
      ;;
    -u|--user)
      USERNAME="$2"
      shift 2
      ;;
    -w|--pass)
      PASSWORD="$2"
      shift 2
      ;;
    -t|--type)
      TYPE="$2"
      shift 2
      ;;
    -c|--count)
      COUNT="$2"
      shift 2
      ;;
    -d|--delay)
      DELAY="$2"
      shift 2
      ;;
    --help)
      echo "RADIUS Server Test Script"
      echo "Usage: ./test_radius.sh [options]"
      echo ""
      echo "Options:"
      echo "  -h, --host HOST       RADIUS server hostname/IP (default: localhost)"
      echo "  -p, --port PORT       RADIUS authentication port (default: 1812)"
      echo "  -s, --secret SECRET   Shared secret (default: testing123)"
      echo "  -u, --user USERNAME   Username to test (default: testuser)"
      echo "  -w, --pass PASSWORD   Password to test (default: password)"
      echo "  -t, --type TYPE       Test type: auth, acct, or both (default: auth)"
      echo "  -c, --count COUNT     Number of requests to send (default: 1)"
      echo "  -d, --delay DELAY     Delay between requests in seconds (default: 0)"
      echo "  --help                Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Check if radtest is installed
if ! command -v radtest &> /dev/null; then
  echo "Error: radtest command not found."
  echo "Please install FreeRADIUS client tools:"
  echo "  - On Ubuntu/Debian: sudo apt-get install freeradius-utils"
  echo "  - On CentOS/RHEL: sudo yum install freeradius-utils"
  echo "  - On macOS: brew install freeradius-server"
  exit 1
fi

# Function to perform auth test
perform_auth_test() {
  local attempt=$1
  echo "🔑 Authentication Test #$attempt - User: $USERNAME"
  
  # Run radtest command
  result=$(radtest "$USERNAME" "$PASSWORD" "$HOST:$PORT" 0 "$SECRET" 2>&1)
  status=$?
  
  # Display result
  if echo "$result" | grep -q "Access-Accept"; then
    echo "✅ Authentication successful!"
    echo "$result" | grep -E "Reply-Message|Framed-IP-Address|Session-Timeout" | sed 's/^/    /'
  elif echo "$result" | grep -q "Access-Reject"; then
    echo "❌ Authentication rejected"
    echo "$result" | grep "Reply-Message" | sed 's/^/    /'
  elif echo "$result" | grep -q "Access-Challenge"; then
    echo "⚠️ Authentication challenge (multi-factor authentication)"
    echo "$result" | grep "Reply-Message" | sed 's/^/    /'
  else
    echo "❓ Unknown response or error"
    echo "$result" | sed 's/^/    /'
  fi
  
  return $status
}

# Function to perform accounting test
perform_acct_test() {
  local attempt=$1
  echo "📊 Accounting Test #$attempt - User: $USERNAME"
  
  # Generate session ID
  session_id="test-session-$RANDOM"
  
  # Start accounting session
  echo "  🔵 Starting accounting session: $session_id"
  acct_result=$(echo "Acct-Status-Type = Start, Acct-Session-Id = \"$session_id\", User-Name = \"$USERNAME\"" | \
    radclient -c 1 -n 3 -r 1 -t 3 "$HOST:$(($PORT + 1))" acct "$SECRET" 2>&1)
  
  if echo "$acct_result" | grep -q "Received response ID"; then
    echo "  ✅ Accounting start successful"
  else
    echo "  ❌ Accounting start failed"
    echo "$acct_result" | sed 's/^/    /'
  fi
  
  # Sleep for a moment
  sleep 2
  
  # Stop accounting session
  echo "  🔴 Stopping accounting session: $session_id"
  acct_result=$(echo "Acct-Status-Type = Stop, Acct-Session-Id = \"$session_id\", User-Name = \"$USERNAME\", Acct-Session-Time = 120" | \
    radclient -c 1 -n 3 -r 1 -t 3 "$HOST:$(($PORT + 1))" acct "$SECRET" 2>&1)
  
  if echo "$acct_result" | grep -q "Received response ID"; then
    echo "  ✅ Accounting stop successful"
  else
    echo "  ❌ Accounting stop failed"
    echo "$acct_result" | sed 's/^/    /'
  fi
}

# Run tests
echo "🚀 Starting RADIUS tests against $HOST:$PORT"
echo "Shared secret: ${SECRET:0:3}***${SECRET: -3}"
echo "Test type: $TYPE"
echo "Count: $COUNT"
echo "------------------------------------"

for ((i=1; i<=COUNT; i++)); do
  if [[ "$TYPE" == "auth" || "$TYPE" == "both" ]]; then
    perform_auth_test $i
  fi
  
  if [[ "$TYPE" == "acct" || "$TYPE" == "both" ]]; then
    perform_acct_test $i
  fi
  
  if [[ $i -lt $COUNT && $DELAY -gt 0 ]]; then
    echo "Waiting ${DELAY}s before next test..."
    sleep $DELAY
  fi
  
  if [[ $i -lt $COUNT ]]; then
    echo "------------------------------------"
  fi
done

echo "------------------------------------"
echo "✨ Tests completed"

#!/usr/bin/env bash

[ ! -f target/debug/shrine ] && echo "Missing executable" && exit 1

mkdir --parents target/smoke-tests
pushd target/smoke-tests > /dev/null || exit 1

readonly GREEN="\033[0;32m"
readonly RED="\033[0;31m"
readonly RESET="\033[0m"
readonly SHRINE="../debug/shrine"
readonly PASSWORD_1="password"
readonly PASSWORD_2="password2"

export RUST_BACKTRACE=1

echo -n "Init shrine ... "
$SHRINE --password="$PASSWORD_1" init --force
output="$($SHRINE --password "$PASSWORD_1" ls)"
[ "$output" != "-> 0 keys found" ] && echo -e "\n${RED}Expected \`-> 0 keys found\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Set secret to password123 ... "
$SHRINE --password "$PASSWORD_1" set secret password123
output="$($SHRINE --password "$PASSWORD_1" ls | tail -n1)"
[ "$output" != "-> 1 keys found" ] && echo -e "\n${RED}Expected \`-> 1 keys found\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Get secret  ... "
output="$($SHRINE --password "$PASSWORD_1" get secret)"
[ "$output" != "password123" ] && echo -e "\n${RED}Expected \`password123\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Change shrine password ... "
output="$($SHRINE --password "$PASSWORD_1" convert --new-password "$PASSWORD_2")"
[ "$output" != "" ] && echo -e "\n${RED}Expected \`\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Get secret  ... "
output="$($SHRINE --password "$PASSWORD_2" get secret)"
[ "$output" != "password123" ] && echo -e "\n${RED}Expected \`password123\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Delete secret ... "
$SHRINE --password="$PASSWORD_2" rm secret
output="$($SHRINE --password "$PASSWORD_2" ls)"
[ "$output" != "-> 0 keys found" ] && echo -e "\n${RED}Expected \`-> 0 keys found\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

popd > /dev/null || exit 1
exit 0
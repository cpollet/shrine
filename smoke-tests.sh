#!/usr/bin/env bash

[ ! -f target/debug/shrine ] && echo "Missing executable" && exit 1

rm -rf target/smoke-tests
mkdir -p target/smoke-tests
pushd target/smoke-tests > /dev/null || exit 1

readonly GREEN="\033[0;32m"
readonly RED="\033[0;31m"
readonly RESET="\033[0m"
readonly SHRINE="$(pwd)/../debug/shrine"
readonly PASSWORD_1="password"
readonly PASSWORD_2="password2"

export RUST_BACKTRACE=1

echo -n "Init shrine ... "
$SHRINE --password "$PASSWORD_1" init --force &>/dev/null
output="$($SHRINE --password "$PASSWORD_1" ls)"
[ "$output" != "total 0" ] && echo -e "\n${RED}Expected \`total 0\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Set secret to password123 ... "
$SHRINE --password "$PASSWORD_1" set secret password123 &>/dev/null
output="$($SHRINE --password "$PASSWORD_1" ls | head -n1)"
[ "$output" != "total 1" ] && echo -e "\n${RED}Expected \`total 1\` got \`$output\`${RESET}" && exit 1
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
$SHRINE --password="$PASSWORD_2" rm secret &>/dev/null
output="$($SHRINE --password "$PASSWORD_2" ls)"
[ "$output" != "total 0" ] && echo -e "\n${RED}Expected \`total 0\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Import from env file ... "
echo -e "key1=val1#comment\n#a comment\n\nkey2=val2==" > env-file
$SHRINE --password="$PASSWORD_2" import env-file &>/dev/null
output="$($SHRINE --password "$PASSWORD_2" get key1)"
[ "$output" != "val1" ] && echo -e "\n${RED}Expected \`val1\` got \`$output\`${RESET}" && exit 1
output="$($SHRINE --password "$PASSWORD_2" get key2)"
[ "$output" != "val2==" ] && echo -e "\n${RED}Expected \`val2==\` got \`$output\`${RESET}" && exit 1
output="$($SHRINE --password "$PASSWORD_2" ls | head -n1)"
[ "$output" != "total 2" ] && echo -e "\n${RED}Expected \`total 2\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Import from env file with prefix ... "
$SHRINE --password="$PASSWORD_2" import env-file --prefix env/ &>/dev/null
output="$($SHRINE --password "$PASSWORD_2" get env/key1)"
[ "$output" != "val1" ] && echo -e "\n${RED}Expected \`val1\` got \`$output\`${RESET}" && exit 1
output="$($SHRINE --password "$PASSWORD_2" get env/key2)"
[ "$output" != "val2==" ] && echo -e "\n${RED}Expected \`val2==\` got \`$output\`${RESET}" && exit 1
output="$($SHRINE --password "$PASSWORD_2" ls | head -n1)"
[ "$output" != "total 4" ] && echo -e "\n${RED}Expected \`total 2\` got \`$output\`${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Create shine in another folder ... "
mkdir another
$SHRINE --password="$PASSWORD_1" --path another init &>/dev/null
[ ! -f another/shrine ] && echo -e "\n${RED}Expected \`another/shrine\` to exist${RESET}" && exit 1
echo -e "${GREEN}ok${RESET}"

echo -n "Create shine with git repo ... "
tmpdir=$(mktemp -d)
pushd "$tmpdir" >/dev/null || exit 1
  $SHRINE --password="$PASSWORD_1" init --git &>/dev/null
  ! git status &>/dev/null && echo -e "\n${RED}Expected \`$(pwd)/with-git\` to be a git repository${RESET}" && exit 1
  ! git log -n1 | grep "Initialize shrine" &>/dev/null && echo -e "\n${RED}Expected \`Initialize shrine\`${RESET}" && exit 1
  echo -e "${GREEN}ok${RESET}"

  echo -n "Update shine with git repo ... "
  $SHRINE --password="$PASSWORD_1" set key val &>/dev/null
  ! git log -n1 | grep "Update shrine" &>/dev/null && echo -e "\n${RED}Expected \`Update shrine\`${RESET}" && exit 1
  echo -e "${GREEN}ok${RESET}"
popd >/dev/null || exit 1
rm -rf "$tmpdir"

echo -n "Disable auto commit ... "
tmpdir=$(mktemp -d)
pushd "$tmpdir" >/dev/null || exit 1
  $SHRINE --password="$PASSWORD_1" init --force --git &>/dev/null
  $SHRINE --password="$PASSWORD_1" config set git.commit.auto false &>/dev/null
  $SHRINE --password="$PASSWORD_1" config set key val &>/dev/null
  output="$(git rev-list HEAD --count)"
  [ "$output" != "2" ] && echo -e "\n${RED}Expected 2 commits, got $output${RESET}" && exit 1
  echo -e "${GREEN}ok${RESET}"
popd >/dev/null || exit 1
rm -rf "$tmpdir"

echo -n "Disable git ... "
tmpdir=$(mktemp -d)
pushd "$tmpdir" >/dev/null || exit 1
  $SHRINE --password="$PASSWORD_1" init --force --git &>/dev/null
  $SHRINE --password="$PASSWORD_1" config set git.enabled false &>/dev/null
  $SHRINE --password="$PASSWORD_1" config set key val &>/dev/null
  output="$(git rev-list HEAD --count)"
  [ "$output" != "2" ] && echo -e "\n${RED}Expected 2 commits, got $output${RESET}" && exit 1
  echo -e "${GREEN}ok${RESET}"
popd >/dev/null || exit 1
rm -rf "$tmpdir"

popd > /dev/null || exit 1
exit 0

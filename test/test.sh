#!/bin/sh
try() {
  expected="$1"
  input="$2"

  cargo run --release "$input" > tmp.s 2>/dev/null
  gcc -o tmp tmp.s
  ./tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $actual"
    echo "$expected expected, but got $actual"
    exit 1
  fi
}

try 0 '0;'
try 42 '42;'
try 21 '5+20-4;'
try 41 ' 12 + 34 - 5 ;'
try 47 "5+6*7;"
try 15 "5*(9-6);"
try 4 "(3+5)/2;"
try 3 'a = 3; a;'
try 14 'a = 3; b = 5 * 6 - 8; a + b / 2;'
try 14 'a = 3; b = 5 * 6 - 8; return a + b / 2;'
try 16 'foo = 5; f = 5 * 6 - 8; return foo + f / 2;'
try 40 'foo= 2; foo= foo * foo; return 10 * foo;'

echo OK

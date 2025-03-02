log="/tmp/stacky_test.log"
touch $log
tail -f $log | ./target/debug/stacky

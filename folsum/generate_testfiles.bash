#!/usr/bin/env bash
rm -rf test_dir;

mkdir -p 'test_dir/subdir_one';
printf 'some textfile content' > 'test_dir/subdir_one/nested_test_file.txt';

printf 'some textfile content' > 'test_dir/test_file_one.txt';
printf 'some textfile content' > 'test_dir/test_file_two.txt';
printf 'Column One, Column Two, Column Three\nItem one, Item two Item three' > 'test_dir/test_file.csv';
zip -rq 'test_dir/test_file.zip' 'test_dir/';

echo 'Generated testfiles in test_dir/'
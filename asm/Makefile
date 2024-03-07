clean: test.o
	rm -f test.o

test.o: test.s
	cc -o test.o test.s

dump: test.o
	objdump -d test.o

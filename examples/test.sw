const one = 1;
const six = 2 * 3;
const number = one + six;

builtin_print__ number == 7;

for x in one..number do
    builtin_print__ x;
end

builtin_print__ #ff77a8;

builtin_print__ { 3.14, 1.2345 * 2.0 };

for nth in 0..20 do
    let a = 0;
    let b = 1;

    for n in 0..nth do
        let c = a + b;
        a = b;
        b = c;
    end

    builtin_print__ a;
end

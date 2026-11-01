i = 0; acc = 0;
while i < 10
    i = i + 1;
    if i == 5
        continue;
    end
    if i > 7
        break;
    end
    acc = acc + i;
end
res = acc;
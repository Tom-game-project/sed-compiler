
def extract_by_index(string:str, index:int) -> chr:
    head, back = string[0], string[1:]
    if index == 0:
        return head
    else:
        return extract_by_index(back, index - 1)

def main():
    ploblem0 = "01234567890123456789"
    index = 0
    while index < 9:
        index += 1;
    cur = extract_by_index(ploblem0, 7)
    print(cur)

main()

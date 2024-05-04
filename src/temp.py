def count_words_in_file(file_path):
    word_count = 0
    try:
        with open(file_path, 'r', encoding='utf-8') as file:
            for line in file:
                words = line.split()  # Splits by any whitespace and discards empty strings
                word_count += len(words)
    except FileNotFoundError:
        print(f"File not found: {file_path}")
    except Exception as e:
        print(f"An error occurred: {e}")
    return word_count

# Usage example
file_path = "text8.txt"  # Replace with the path to your file
total_words = count_words_in_file(file_path)
print(f"Total words in the file: {total_words}")
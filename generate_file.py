import random
import os

# Word list for generating realistic text
WORD_LIST = [
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "I",
    "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
    "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
    "or", "an", "will", "my", "one", "all", "would", "there", "their", "what",
    "so", "up", "out", "if", "about", "who", "get", "which", "go", "me",
    "when", "make", "can", "like", "time", "no", "just", "him", "know", "take",
    "people", "into", "year", "your", "good", "some", "could", "them", "see", "other",
    "than", "then", "now", "look", "only", "come", "its", "over", "think", "also",
    "back", "after", "use", "two", "how", "our", "work", "first", "well", "way",
    "even", "new", "want", "because", "any", "these", "give", "day", "most", "us",
    "is", "are", "was", "were", "has", "had", "been", "being", "have", "having",
    "does", "did", "doing", "done", "said", "says", "saying", "went", "gone", "going"
]

PUNCTUATION = [",", ".", ";", ":", "!", "?", "-", "'", '"']

def generate_line(min_words=1, max_words=20):
    word_count = random.randint(min_words, max_words)
    line = []
    for _ in range(word_count):
        word = random.choice(WORD_LIST)
        # Randomly add punctuation
        if random.random() < 0.1:
            word += random.choice(PUNCTUATION)
        line.append(word)
    return " ".join(line)

def generate_file(filename, line_count=1_000_000):
    with open(filename, "w") as f:
        for i in range(line_count):
            line = generate_line()
            # Add some empty lines
            if random.random() < 0.01:
                line = ""
            # Add some very long lines
            if random.random() < 0.005:
                line = generate_line(50, 100)
            f.write(line + "\n")
            # Progress indicator
            if i % 100_000 == 0:
                print(f"Generated {i:,} lines...")

if __name__ == "__main__":
    filename = "test_large.txt"
    print(f"Generating test file '{filename}' with 1,000,000 lines...")
    generate_file(filename)
    file_size = os.path.getsize(filename) / (1024 * 1024)
    print(f"Done! File size: {file_size:.2f} MB")
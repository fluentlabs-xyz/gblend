# @version ^0.3.0

# Create a string variable that can store a maximum of 100 characters
greet: public(String[100])

@external
def __init__():
    self.greet = "Hello, World!"

import math

value_min = -70.0
value_max = 0.0
parts = 10

sqrt_value_min = math.sqrt(abs(value_min))
sqrt_value_max = math.sqrt(abs(value_max))

print(sqrt_value_min, sqrt_value_max)

sqrt_width = sqrt_value_min - sqrt_value_max

for i in range(0, parts):
    print(pow(sqrt_width - (sqrt_width / parts) * i, 2.0))



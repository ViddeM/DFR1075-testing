static int PIN = 15;

void setup() {

  pinMode(PIN, OUTPUT);
  digitalWrite(PIN, HIGH); // Turn on the LED
  
  Serial.begin(115200);
  delay(3000); // Wait for USB CDC enumeration

  // Set ADC resolution to 12 bits (0–4095)
  analogReadResolution(12);
}

void loop() {
  // Read raw ADC value on pin 0
  int analogValue = analogRead(0);
  int analogVolts = analogReadMilliVolts(0);

  // Print raw ADC value
  Serial.print("ADC analog value = ");
  Serial.println(analogValue);

  // Print measured voltage (in millivolts)
  Serial.print("ADC millivolts value = ");
  Serial.print(analogVolts);
  Serial.println(" mV");

  // Apply correction coefficient based on hardware voltage divider
  Serial.print("BAT millivolts value = ");
  Serial.print(analogVolts * 2);
  Serial.println(" mV");

  Serial.println("--------------");

  digitalWrite(PIN, HIGH); // Turn on the LED
  delay(750);

  digitalWrite(PIN, LOW); // Turn off the LED
  delay(250);
}

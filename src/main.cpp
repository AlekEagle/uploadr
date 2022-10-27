#include "util/not-notify/not-notify.hpp"
#include <iostream>
#include "util/clipboard/clipboard.hpp"

using namespace std;

int main(int argc, char **argv)
{
  // Initialize the notification system
  if (!initNotify("Uploadr"))
  {
    cout << "Failed to initialize libnotify" << endl;
    return 1;
  }
}
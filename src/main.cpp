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

  // Create a notification
  NotNotification *notification = new NotNotification("Uploadr", Clipboard::getClipboard().c_str(), "uploadr");
  notification->show();
}
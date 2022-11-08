#pragma once
#include <chrono>
#include <libnotify/notify.h>
#include <string>
#include <thread>

// Man I hate Glib, but I'm too lazy to use pure DBus calls, and even if I did
// I'd still have to figure out how to handle notification action callbacks.
// So Glib it is!

namespace NotNotify {
  // Your standard notify init and uninit functions
  bool init(std::string app_name);
  bool init(const char *app_name);
  void uninit();

  // The meat and bones of this whole thing, the notification class
  class Notification {
    private:
      NotifyNotification *n;
      void initSelf();

    public:
      Notification(std::string summary, std::string body, std::string icon);
      Notification(std::string summary, std::string body);
      Notification(std::string summary);
      Notification();
      ~Notification();
      void show();
      void close();
      void updateNotification(
        std::string summary, std::string body, std::string icon
      );
      void updateNotification(std::string summary, std::string body);
      void updateNotification(std::string summary);
      void updateIcon(std::string icon);
      void addAction(
        std::string action, std::string label,
        void (*callback)(NotifyNotification *n, char *action, void *data),
        void *data
      );
      void clearActions();
      void setUrgency(NotifyUrgency urgency);
      void setCategory(std::string category);
      void setTimeout(int timeout);
      void makePersistent();
      void defaultTimeout();
      void runLoop();
  };
} // namespace NotNotify
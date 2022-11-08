#include "not-notify.hpp"
#include <iostream>
#include <libnotify/notify.h>
#include <string>

typedef void (*LambdaType)(void *);

namespace NotNotify {
  // Glib main loop
  GMainLoop *loop = g_main_loop_new(NULL, FALSE);
  bool init(std::string app_name) {
    return notify_init(app_name.c_str());
  }

  bool init(const char *app_name) {
    return notify_init(app_name);
  }

  void uninit() {
    notify_uninit();
  }

  Notification::Notification(
    std::string summary, std::string body, std::string icon
  ) {
    n = notify_notification_new(summary.c_str(), body.c_str(), icon.c_str());
  }

  Notification::Notification(std::string summary, std::string body) {
    n = notify_notification_new(summary.c_str(), body.c_str(), NULL);
  }

  Notification::Notification(std::string summary) {
    n = notify_notification_new(summary.c_str(), NULL, NULL);
  }

  Notification::Notification() {
    n = notify_notification_new(NULL, NULL, NULL);
  }

  // Close the notification, quit the main loop, and free the main loop
  // We don't need to unref the notification because it's done automatically
  // when it's closed
  Notification::~Notification() {
    notify_notification_close(n, NULL);
  }

  void Notification::show() {
    notify_notification_show(n, NULL);
  }

  void Notification::close() {
    notify_notification_close(n, NULL);
  }

  void Notification::updateNotification(
    std::string summary, std::string body, std::string icon
  ) {
    notify_notification_update(n, summary.c_str(), body.c_str(), icon.c_str());
  }

  void Notification::updateNotification(std::string summary, std::string body) {
    notify_notification_update(n, summary.c_str(), body.c_str(), NULL);
  }

  void Notification::updateNotification(std::string summary) {
    notify_notification_update(n, summary.c_str(), "", NULL);
  }

  void Notification::updateIcon(std::string icon) {
    notify_notification_set_icon_from_pixbuf(
      n, gdk_pixbuf_new_from_file(icon.c_str(), NULL)
    );
  }

  void Notification::addAction(
    std::string action, std::string label,
    void (*callback)(NotifyNotification *n, char *action, void *data),
    void *data
  ) {
    notify_notification_add_action(
      n, action.c_str(), label.c_str(), callback, data, NULL
    );
  }

  void Notification::clearActions() {
    notify_notification_clear_actions(n);
  }

  void Notification::setUrgency(NotifyUrgency urgency) {
    notify_notification_set_urgency(n, urgency);
  }

  void Notification::setCategory(std::string category) {
    notify_notification_set_category(n, category.c_str());
  }

  void Notification::setTimeout(int timeout) {
    notify_notification_set_timeout(n, timeout);
  }

  void Notification::makePersistent() {
    notify_notification_set_timeout(n, NOTIFY_EXPIRES_NEVER);
  }

  void Notification::defaultTimeout() {
    notify_notification_set_timeout(n, NOTIFY_EXPIRES_DEFAULT);
  }

  void Notification::runLoop() {
    auto on_close = []() {
      g_main_loop_quit(loop);
      std::cout << "Closed" << std::endl;
    };
    g_signal_connect(n, "closed", G_CALLBACK(on_close), NULL);
    return g_main_loop_run(loop);
  }
} // namespace NotNotify
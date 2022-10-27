#include <libnotify/notify.h>
#include "not-notify.hpp"


NotNotification::NotNotification(const char *summary, const char *body, const char *icon) {
  __notification = notify_notification_new(summary, body, icon);
}
NotNotification::~NotNotification() {
  g_object_unref(G_OBJECT(__notification));
}
NotNotification *NotNotification::setUrgency(NotifyUrgency urgency) {
  notify_notification_set_urgency(__notification, urgency);
  return this;
}
NotNotification *NotNotification::setTimeout(int timeout) {
  notify_notification_set_timeout(__notification, timeout);
  return this;
}
NotNotification *NotNotification::setCategory(const char *category) {
  notify_notification_set_category(__notification, category);
  return this;
}
NotNotification *NotNotification::setAppName(const char *appname) {
  notify_notification_set_app_name(__notification, appname);
  return this;
}
NotNotification *NotNotification::addAction(const char *action, const char *label, NotifyActionCallback callback, void *user_data) {
  notify_notification_add_action(__notification, action, label, callback, user_data, NULL);
  return this;
}
NotNotification *NotNotification::show() {
  notify_notification_show(__notification, NULL);
  return this;
}
NotNotification *NotNotification::close() {
  notify_notification_close(__notification, NULL);
  return this;
}
NotNotification *NotNotification::update(const char *summary, const char *body, const char *icon) {
  notify_notification_update(__notification, summary, body, icon);
  return this;
}
gint NotNotification::getClosedReason() {
  return notify_notification_get_closed_reason(__notification);
}

bool initNotify(const char *appName) {
  return notify_init(appName);
}
#pragma once
#include <libnotify/notify.h>

bool initNotify(const char *appName);

class NotNotification
{
  public:
    NotifyNotification *__notification;

    NotNotification(const char *summary, const char *body, const char *icon);
    ~NotNotification();
    NotNotification *setUrgency(NotifyUrgency urgency);
    NotNotification *setTimeout(int timeout);
    NotNotification *setCategory(const char *category);
    NotNotification *setAppName(const char *appname);
    NotNotification *addAction(const char *action, const char *label, NotifyActionCallback callback, void *user_data);
    NotNotification *show();
    NotNotification *close();
    NotNotification *update(const char *summary, const char *body, const char *icon);
    gint getClosedReason();
};
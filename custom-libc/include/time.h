#ifndef _TIME_H
#define _TIME_H

typedef long time_t;
typedef int clockid_t;

time_t time(time_t* tloc);
int gettimeofday(void* tv, void* tz);

#endif

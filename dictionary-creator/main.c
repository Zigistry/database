#include <sqlite3.h>
#include <stdio.h>


int main() {
  sqlite3 *db_ptr = NULL;

  if(sqlite3_open("zigistry.db", &db_ptr)) {
    printf("Unable to open db");
    return 1;
  } else {
    printf("Database loaded");
  }

  return 0;
}

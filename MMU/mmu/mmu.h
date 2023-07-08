#ifndef __MAIN_H
#define __MAIN_H

using namespace std;
#include<iostream>


typedef struct line {
	bool valid;
	bool ref;
	bool dirty;
	int page_number;
	int frame_number;
} line;

typedef struct entry {
	bool valid;
	bool dirty;
	int frame_number;
	int disk_address;
}entry;

typedef struct TCB {
	string name;
	entry* page_table;
}TCB;

#endif


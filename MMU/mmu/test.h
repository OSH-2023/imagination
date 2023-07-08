#ifndef __TEST_H
#define __TEST_H

#include <iostream>
#include <time.h>
#include <stdlib.h>
#include "mmu_simulate.h"

void test_add();
void test_sort();
void swap(int i, int j);
void bubbleSort();
float get_hit_rate(int hit, int miss);
void test_sort_merge();
void mergeSort(int left, int right);
void merge(int left, int mid, int right);

#endif


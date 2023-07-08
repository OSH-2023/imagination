#include "test.h"

void creat_random(int begin_num, int end_num)
{
	srand((unsigned)time(NULL));

	for (int i = 0; i < disk_size; i++) {
		disk[i] = (rand() % end_num) + begin_num;
	}
}
void test_add()
{
	int res = 0;
	TCB *tcb = (TCB *)malloc(sizeof(TCB));
	for (int i = 0; i < disk_size; i++)
	{
		disk[i] = 1;
	}
	page_table_init(tcb);
	currentTCB = tcb;
	LRU_list_init(LRU_list);


	for (int i = 0; i < virtual_space; i++)
	{
		res += memory[address_map(i, memory_operation::read)];
	}
	cout << res << endl;
}

void test_sort()
{
	TCB *tcb = (TCB *)malloc(sizeof(TCB));
	creat_random(1, 10000);
	page_table_init(tcb);
	currentTCB = tcb;
	LRU_list_init(LRU_list);
	bubbleSort();
#if(1 == useTLB)
	cout << "TLB的命中率为：" << get_hit_rate(TLB_hit, TLB_miss) << endl;
#endif
	cout << "memory的命中率为：" << get_hit_rate(memory_hit, memory_miss) << endl;
	cout << "运行时间为：" << time_cost << "s" << endl;
	cout << "over" << endl;
	cout << "over" << endl;
}

void swap(int i, int j) 
{
	int temp = read_memory(i);
	write_memory(i, read_memory(j));
	write_memory(j, temp);
}

void bubbleSort() 
{
	int i, j;
	for (i = 0; i < virtual_space - 1; i++) {
		for (j = 0; j < virtual_space - i - 1; j++) {
			if (read_memory(j) > read_memory(j + 1)) {
				swap(j, j + 1);
			}
		}
	}
	wirte_back();
}
float get_hit_rate(int hit, int miss)
{
	return ((float)hit) / (hit + miss);
}

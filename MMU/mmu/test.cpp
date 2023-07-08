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
#if(0 == ReplacementStrategy)
	LRU_list_init(LRU_list);
#endif
#if(1 == ReplacementStrategy)
	FIFO_list_init(FIFO_list);
#endif
	bubbleSort();
	wirte_back();
#if(1 == useTLB)
	cout << "TLB��������Ϊ��" << get_hit_rate(TLB_hit, TLB_miss) << endl;
#endif
	cout << "memory��������Ϊ��" << get_hit_rate(memory_hit, memory_miss) << endl;
	cout << "����ʱ��Ϊ��" << time_cost << "s" << endl;
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
}
float get_hit_rate(int hit, int miss)
{
	return ((float)hit) / (hit + miss);
}

void test_sort_merge()
{
	TCB *tcb = (TCB *)malloc(sizeof(TCB));
	creat_random(1, 10000);
	page_table_init(tcb);
	currentTCB = tcb;
#if(0 == ReplacementStrategy)
	LRU_list_init(LRU_list);
#endif
#if(1 == ReplacementStrategy)
	FIFO_list_init(FIFO_list);
#endif
	mergeSort(0, virtual_space - 1);
	wirte_back();
#if(1 == useTLB)
	cout << "TLB��������Ϊ��" << get_hit_rate(TLB_hit, TLB_miss) << endl;
#endif
	cout << "memory��������Ϊ��" << get_hit_rate(memory_hit, memory_miss) << endl;
	cout << "����ʱ��Ϊ��" << time_cost << "s" << endl;
	cout << "over" << endl;

}

void merge(int left, int mid, int right) {
	int i, j, k;
	int n1 = mid - left + 1;
	int n2 = right - mid;

	// ������ʱ����
	int *L = (int*)malloc(sizeof(int) * n1);
	int *R = (int*)malloc(sizeof(int) * n2);

	// �������ݵ���ʱ���� L[] �� R[]
	for (i = 0; i < n1; i++)
		L[i] = read_memory(left + i);
	for (j = 0; j < n2; j++)
		R[j] = read_memory(mid + 1 + j);

	// �鲢��ʱ���鵽 arr[left..right]
	i = 0;
	j = 0;
	k = left;
	while (i < n1 && j < n2) {
		if (L[i] <= R[j]) {
			write_memory(k, L[i]);
			i++;
		}
		else {
			write_memory(k, R[j]);
			j++;
		}
		k++;
	}

	// ���� L[] ��ʣ��Ԫ��
	while (i < n1) {
		write_memory(k, L[i]);
		i++;
		k++;
	}

	// ���� R[] ��ʣ��Ԫ��
	while (j < n2) {
		write_memory(k, R[j]);
		j++;
		k++;
	}

	free(L);
	free(R);
}

// �͵ع鲢����
void mergeSort(int left, int right)
{
	if (left < right) {
		int mid = left + (right - left) / 2;

		// ����������������
		mergeSort(left, mid);
		// ���Ҳ������������
		mergeSort(mid + 1, right);

		// �ϲ�����������������
		merge(left, mid, right);
	}
}
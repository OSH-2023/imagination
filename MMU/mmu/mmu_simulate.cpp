#include "mmu_simulate.h"
int memory[memory_size];
int disk[disk_size];
TCB *currentTCB;
line TLB[TLB_size];

int replacement_number_FIFO;
float time_cost;
long int TLB_hit;
long int TLB_miss;
long int memory_hit;
long int memory_miss;

int read_to_memory(int memory_frame, int disk_start_address)
{
	int memory_address, disk_address;
	for (int i = 0; i < page_size; i++)
	{
		memory_address = memory_frame * page_size + i;
		disk_address = disk_start_address + i;
		memory[memory_address] = disk[disk_address];
	}
	time_cost += time_disk_access;
	return 1;
}

int write_to_disk(int memory_frame, int disk_start_address)
{
	int memory_address, disk_address;
	for (int i = 0; i < page_size; i++)
	{
		memory_address = memory_frame * page_size + i;
		disk_address = disk_start_address + i;
		disk[disk_address] = memory[memory_address];
	}
	time_cost += time_disk_access;
	return 1;
}

void page_table_init(TCB *tcb)
{
	tcb->page_table = (entry*)malloc(page_table_size * sizeof(entry));
	for (int i = 0; i < page_table_size; i++)
	{
		tcb->page_table[i].dirty = false;
		tcb->page_table[i].valid = false;
		tcb->page_table[i].frame_number = 0;
		tcb->page_table[i].disk_address = start_address + page_size * i;
	}
	//将外存中的一部分数据放入内存
	for (int i = 0; i < memory_frame_size; i++)
	{
		read_to_memory(i, tcb->page_table[i].disk_address);
		tcb->page_table[i].frame_number = i;
		tcb->page_table[i].valid = true;
	}
}

int address_map(int virtual_address, memory_operation operation)//operation表示读或写
{
	int page_number = virtual_address / page_size;
	int offset = virtual_address % page_size;
	int physical_address;
#if(1 == useTLB)
	if ((physical_address = TLB_search(virtual_address, operation)) != -1)//快表命中
	{
		TLB_hit++;
		memory_hit++;
		return physical_address;
	}
	TLB_miss++;
#endif
	//查询慢表
	time_cost += time_memory_access;
	if (!currentTCB->page_table[page_number].valid)
	{
		pageFault((currentTCB->page_table) + page_number, page_number);
		memory_miss++;
	}
	else memory_hit++;
	physical_address = currentTCB->page_table[page_number].frame_number * page_size + offset;
#if(0 == ReplacementStrategy)
	pMovetoFirst(LRU_list, currentTCB->page_table[page_number].frame_number);
#endif
	if (operation == memory_operation::write)
	{
		currentTCB->page_table[page_number].dirty = true;
	}
#if(1 == useTLB)
	TLB_update(page_number, currentTCB->page_table[page_number].frame_number);//更新快表
#endif
	return physical_address;
	
}

void pageFault(entry * faultPage, int page_number)
{
#if(0 == ReplacementStrategy)
	LINKNODE endNode = GetEndNode(LRU_list);
#endif
#if(1 == ReplacementStrategy)
	LINKNODE endNode = FIFO_list;
#endif
#if(1 == useTLB)
	if (endNode->task_belonging == currentTCB)//这种情况下可能需要回写快表中的脏位，以及修改快表内容
	{
		for (int i = 0; i < TLB_size; i++)
		{
			if (TLB[i].valid == true && TLB[i].page_number == endNode->page_number)//需要调出的页还在快表里
			{
				TLB[i].valid = false;
				if (TLB[i].dirty == true)
				{
					((currentTCB->page_table) + (endNode->page_number))->dirty = true;//将快表中的dirty写回慢表
				}
				break;
			}
		}
	}
#endif
	//换出
	if (((endNode->task_belonging->page_table) + (endNode->page_number))->dirty == true)
	{
		int disk_address_out = ((endNode->task_belonging->page_table) + (endNode->page_number))->disk_address;
		write_to_disk(endNode->frame_number, disk_address_out);
	}
	((endNode->task_belonging->page_table) + (endNode->page_number))->valid = false;
	//换入
	int disk_address_in = faultPage->disk_address;
	read_to_memory(endNode->frame_number, disk_address_in);
	faultPage->dirty = false;
	faultPage->valid = true;
	faultPage->frame_number = endNode->frame_number;
	//更新节点对应帧的信息
	endNode->task_belonging = currentTCB;
	endNode->page_number = page_number;
#if(1 == ReplacementStrategy)
	FIFO_list = FIFO_list->next;
#endif
}

void LRU_list_init(LINKNODE &list)
{
	LINKNODE s;
	pInitList(list);
	for (int i = 0; i < memory_frame_size; i++)
	{
		s = (LINKNODE)malloc(sizeof(NODE));
		s->task_belonging = currentTCB;
		s->page_number = i;
		s->frame_number = i;
		pInsertElem(list, s, 1);
	}
}

int read_memory(int virtual_address)
{
	int physical_address = address_map(virtual_address, memory_operation::read);
	int data = memory[physical_address];
	time_cost += time_cach_access;
	return data;
}

void write_memory(int virtual_address, int data)
{
	int physical_address = address_map(virtual_address, memory_operation::write);
	memory[physical_address] = data;
	time_cost += time_cach_access;
}



int TLB_search(int virtual_address, memory_operation operation)//快表命中返回物理地址，否则返回-1
{
	int page_number = virtual_address / page_size;
	int offset = virtual_address % page_size;
	int physical_address = -1;//如果返回了-1，说明没有匹配到
	for (int i = 0; i < TLB_size; i++)
	{
		if (TLB[i].page_number == page_number && TLB[i].valid == true)//成功匹配
		{
			TLB[i].ref = true;
			if (operation == memory_operation::write)
			{
				TLB[i].dirty = true;
			}
			//cout << i << " ";
			physical_address = TLB[i].frame_number * page_size + offset;
			break;
		}
	}
	time_cost += time_TLB_access;
	return physical_address;
}

int TLB_update(int page_number, int frame_number)
{
	for (int i = 0; i < TLB_size; i++)//寻找应该被替换的行
	{
		if (TLB[i].valid == false)
		{
			TLB[i].page_number = page_number;
			TLB[i].frame_number = frame_number;
			TLB[i].dirty = false;
			TLB[i].ref = true;
			TLB[i].valid = true;
			return 1;
		}
	}
	for (int i = 0; i < TLB_size; i++)//寻找应该被替换的行
	{
		if (TLB[i].valid == true && TLB[i].ref == true && i != TLB_size - 1)
		{
			TLB[i].ref = false;
		}
		else
		{
			if (TLB[i].valid == true && TLB[i].dirty == true)
			{
				((currentTCB->page_table) + (TLB[i].page_number))->dirty = true;//将快表中的dirty写回慢表
			}
			TLB[i].page_number = page_number;
			TLB[i].frame_number = frame_number;
			TLB[i].dirty = false;
			TLB[i].ref = true;
			TLB[i].valid = true;
			break;
		}
	}
	return 1;
}

void wirte_back()
{
	for (int i = 0; i < TLB_size; i++)
	{
		if (TLB[i].valid == true && TLB[i].dirty == true)
		{
			((currentTCB->page_table) + (TLB[i].page_number))->dirty = true;//将快表中的dirty写回慢表
		}
	}
	
	for (int i = 0; i < memory_frame_size; i++)
	{
		if (((currentTCB->page_table) + i)->valid == true && ((currentTCB->page_table) + i)->dirty == true)
		{
			write_to_disk(((currentTCB->page_table) + i)->frame_number, ((currentTCB->page_table) + i)->disk_address);
		}
	}
	
}

void FIFO_list_init(LINKNODE &list)
{
	LINKNODE s, p;
	pInitList(list);
	for (int i = 0; i < memory_frame_size; i++)
	{
		s = (LINKNODE)malloc(sizeof(NODE));
		s->task_belonging = currentTCB;
		s->page_number = i;
		s->frame_number = i;
		pInsertElem(list, s, 1);
	}
	p = GetEndNode(list);
	p->next = list->next;
	list = list->next;
}



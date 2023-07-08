#pragma once
#ifndef __MMU_SIMULATE_H
#define __MMU_SIMULATE_H

using namespace std;
#include<iostream>
#include"list.h"
#include "mmu.h"

#define useTLB	1
#define ReplacementStrategy 0	//0-LRU��1-FIFO

#define times 8
#define page_size (128 * times)  //ҳ��С
#define memory_size (1024 * times)   //�ڴ��С
#define memory_frame_size (memory_size / page_size)   //�ڴ�ҳ����
#define disk_size (4096 * times) //����С
#define virtual_space (2048 * times) //�����ַ�ռ�
#define page_table_size (virtual_space / page_size) //ҳ���С
#define TLB_size (4 * times) //TLB��С
#define time_TLB_access 1e-9   //����TLBʱ��
#define time_memory_access 1e-7  //����memoryʱ��  
#define	time_cach_access 1e-7	//����cachʱ��
#define time_disk_access 1e-3    //����Ӳ��ʱ��
#define start_address 0 //�������ݴ����ʼ��ַ

extern int memory[memory_size];
extern int disk[disk_size];
extern line TLB[TLB_size];
extern TCB *currentTCB;
extern long int TLB_hit;
extern long int TLB_miss;
extern long int memory_hit;
extern long int memory_miss;
extern int replacement_number_FIFO;
extern float time_cost;

enum memory_operation {
	read = 0,
	write = 1
};

void page_table_init(TCB *tcb);
void FIFO_list_init(LINKNODE &list);
int address_map(int virtual_address, memory_operation operation);
int read_to_memory(int memory_frame, int disk_frame);
int write_to_disk(int memory_frame, int disk_start_address);
void pageFault(entry * faultPage, int page_number);
void LRU_list_init(LINKNODE &list);
int read_memory(int virtual_address);
void write_memory(int virtual_address, int data);
int TLB_search(int virtual_address, memory_operation operation);
int TLB_update(int page_number, int frame_number);
void wirte_back();
#endif

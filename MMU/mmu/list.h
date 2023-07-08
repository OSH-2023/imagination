#ifndef __LIST_H
#define __LIST_H

using namespace std;
#include<iostream>
#include "mmu.h"

typedef struct NODE {
	int frame_number;
	struct NODE* next;
	int page_number;
	TCB *task_belonging;
}NODE, *LINKNODE;

void pInitList(LINKNODE &pHead);//链表初始化
int pInsertElem(LINKNODE &pHead, LINKNODE s, int posi); //插入一个节点，节点位置从1开始，头节点位置为0
int pMovetoFirst(LINKNODE &pHead, int e); //从将元素e从当前位置移动到第一个位置，失败
LINKNODE GetEndNode(LINKNODE pHead); //得到末尾节点的指针
int pTraverseList(LINKNODE pHead);

extern LINKNODE LRU_list;
extern LINKNODE FIFO_list;
#endif

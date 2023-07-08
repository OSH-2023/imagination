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

void pInitList(LINKNODE &pHead);//�����ʼ��
int pInsertElem(LINKNODE &pHead, LINKNODE s, int posi); //����һ���ڵ㣬�ڵ�λ�ô�1��ʼ��ͷ�ڵ�λ��Ϊ0
int pMovetoFirst(LINKNODE &pHead, int e); //�ӽ�Ԫ��e�ӵ�ǰλ���ƶ�����һ��λ�ã�ʧ��
LINKNODE GetEndNode(LINKNODE pHead); //�õ�ĩβ�ڵ��ָ��
int pTraverseList(LINKNODE pHead);

extern LINKNODE LRU_list;
extern LINKNODE FIFO_list;
#endif

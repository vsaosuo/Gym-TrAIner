#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>
#include "../address_map_arm.h"

/* Prototypes for functions used to access physical memory addresses */
int open_physical (int);
void * map_physical (int, unsigned int, unsigned int);
void close_physical (int);
int unmap_physical (void *, unsigned int);

#define ALT_UP_RS232_DATA_DATA_MSK			(0x000001FF)
#define ALT_UP_RS232_DATA_DATA_OFST			(0)
#define ALT_UP_RS232_DATA_RVALID_MSK		(0x00008000)
#define ALT_UP_RS232_DATA_RVALID_OFST		(15)
#define ALT_UP_RS232_DATA_RAVAIL_MSK		(0xFFFF0000)
#define ALT_UP_RS232_DATA_RAVAIL_OFST		(16)

void main()
{
   volatile int * uart_ptr;
   uint8_t rx_data, availBuff;
   uint32_t data_reg;

   int fd = -1;               // used to open /dev/mem for access to physical addresses
   void *LW_virtual;          // used to map physical addresses for the light-weight bridge

   // Create virtual memory access to the FPGA light-weight bridge
   if ((fd = open_physical (fd)) == -1)
      return (-1);
   if ((LW_virtual = map_physical (fd, LW_BRIDGE_BASE, LW_BRIDGE_SPAN)) == NULL)
      return (-1);

   // Set virtual address pointer to I/O port
   uart_ptr  = (unsigned int *) (LW_virtual + 0x1020);
    printf("tets");
   while(1){
    data_reg = *uart_ptr;
    printf("0x%x\n",data_reg);
    availBuff = (data_reg & ALT_UP_RS232_DATA_RAVAIL_MSK) >> ALT_UP_RS232_DATA_RAVAIL_OFST;
    if( (data_reg & ALT_UP_RS232_DATA_RVALID_MSK) >> ALT_UP_RS232_DATA_RVALID_OFST ){
        rx_data = (data_reg & ALT_UP_RS232_DATA_DATA_MSK) >> ALT_UP_RS232_DATA_DATA_OFST;
        printf("Read: 0x%x with 0x%x data left\n", rx_data , availBuff);
    }
    usleep(250000);
   }

   unmap_physical (LW_virtual, LW_BRIDGE_SPAN);   // release the physical-memory mapping
   close_physical (fd);   // close /dev/mem
   return 0;
}

// Open /dev/mem, if not already done, to give access to physical addresses
int open_physical (int fd)
{
   if (fd == -1)
      if ((fd = open( "/dev/mem", (O_RDWR | O_SYNC))) == -1)
      {
         printf ("ERROR: could not open \"/dev/mem\"...\n");
         return (-1);
      }
   return fd;
}

// Close /dev/mem to give access to physical addresses
void close_physical (int fd)
{
   close (fd);
}

/*
 * Establish a virtual address mapping for the physical addresses starting at base, and
 * extending by span bytes.
 */
void* map_physical(int fd, unsigned int base, unsigned int span)
{
   void *virtual_base;

   // Get a mapping from physical addresses to virtual addresses
   virtual_base = mmap (NULL, span, (PROT_READ | PROT_WRITE), MAP_SHARED, fd, base);
   if (virtual_base == MAP_FAILED)
   {
      printf ("ERROR: mmap() failed...\n");
      close (fd);
      return (NULL);
   }
   return virtual_base;
}

/*
 * Close the previously-opened virtual address mapping
 */
int unmap_physical(void * virtual_base, unsigned int span)
{
   if (munmap (virtual_base, span) != 0)
   {
      printf ("ERROR: munmap() failed...\n");
      return (-1);
   }
   return 0;
}



#include "server.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "header.h"

int main(void)
{
    int result = 0;
    uint32_t ipv4       = INADDR_ANY;
    unsigned int port   = htons(PORT);

    struct sockaddr_in sock = socket_init(&port, &ipv4);
    int file_descriptor = socket(PF_INET, SOCK_STREAM, 0);

    bind_c(&sock, file_descriptor);
    listen_c(&sock, file_descriptor);




    for(;;){
        struct sockaddr_in client = {0};
        socklen_t client_len = sizeof(client);

        int client_fd = accept_c(&client, &client_len, file_descriptor);

        
        printf("Client connected! fd: %d\n", client_fd);

        pthread_t thr;

        pthread_create(&thr, NULL, worker, (void *)(intptr_t)client_fd);


        //receive_message(buffer, bytes_received, client_fd);


    }
    return 0;
}
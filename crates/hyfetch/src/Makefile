NAME = pride_month

SRC = pride_month.rs

RC = rustc

all: $(NAME)

$(NAME): $(SRC)
	$(RC) $(SRC)

RESET = reset && make re && reset

SILENT_FCLEAN = make -s fclean

fclean:
	rm -rf $(NAME)

re: fclean all

run:
	$(RESET) && ./$(NAME) && $(SILENT_FCLEAN)

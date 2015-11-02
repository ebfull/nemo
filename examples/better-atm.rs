proto!(Atm, Start = {
    Recv String,
    AtmMenu = Accept {
        AtmDeposit = {
            Recv u64,
            Send u64,
            Goto AtmMenu
        },
        AtmWithdraw = {
            Recv u64,
            Send bool,
            Goto AtmMenu
        },
        AtmGetBalance = {
            Send u64,
            Goto AtmMenu
        },
        End
    }
});

fn main() {
    struct Context {
        balance: usize
    }

    let (server, client) = Blocking::new::<Start>();

    let mut ctx = Context { balance: 0 };

    server.context(&mut ctx);

    let mut server = server.recv(|account_number, this| {
        fn atm_menu(this: Channel<Context, Blocking, alias!(AtmMenu)>)
                    -> Defer<Context, Blocking> {

            this.accept::<AtmDeposit>(|this| {
                    this.recv(|val, this| {
                        thix.ctx.balance += val;

                        this.send(this.ctx.balance)
                            .goto(atm_menu)
                    })
                })
                .or::<AtmWithdraw>(|this| {
                    this.recv(|val, this| {
                        if this.ctx.balance >= val {
                            this.ctx.balance -= val;
                            this.send(true)
                                .goto(atm_menu)
                        } else {
                            this.send(false)
                                .goto(atm_menu)
                        }
                    })
                })
                .or::<AtmGetBalance>(|this| {
                    this.send(this.ctx.balance)
                        .goto()
                        .defer(atm_menu)
                })
                .or::<End>(|this| {
                    println!("Closing connection.");
                    this.close()
                })
        }

        atm_menu(this)
    });

    loop {
        match server.with(&mut ctx) {
            Closed => {break;},
            Receiving => {},
            Blocked => {},
        }
    }
}
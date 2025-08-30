// SystemVerilog output equivalent of FSMGen VHDL generation
// This shows how the same internal representation could target SystemVerilog

// Instead of VHDL entity:
// ENTITY mipicsi2_txdcore_hs IS
//   PORT (
//     clk    : IN  STD_LOGIC;
//     rstn   : IN  STD_LOGIC;
//     ...
//   );
// END ENTITY;

module mipicsi2_txdcore_hs (
    input  logic        clk,
    input  logic        rstn,
    input  logic [7:0]  data_in,
    output logic        valid_out,
    output logic        ready
);

// Instead of VHDL state type:
// TYPE state_type IS (IDLE, ACTIVE, WAIT_ACK);

typedef enum logic [1:0] {
    IDLE    = 2'b00,
    ACTIVE  = 2'b01,
    WAIT_ACK = 2'b10
} state_type;

state_type current_state, next_state;

// Instead of VHDL signals:
// SIGNAL counter : STD_LOGIC_VECTOR(7 DOWNTO 0);

logic [7:0] counter;
logic       counter_en;
logic       counter_rst;

// Instead of VHDL process:
// process(clk, rstn)
// begin
//   if rstn = '0' then
//     current_state <= IDLE;
//   elsif rising_edge(clk) then
//     current_state <= next_state;
//   end if;
// end process;

always_ff @(posedge clk or negedge rstn) begin
    if (!rstn) begin
        current_state <= IDLE;
        counter <= '0;
    end else begin
        current_state <= next_state;
        if (counter_rst)
            counter <= '0;
        else if (counter_en)
            counter <= counter + 1'b1;
    end
end

// Combinational logic for next state and outputs
always_comb begin
    next_state = current_state;
    counter_en = 1'b0;
    counter_rst = 1'b0;
    valid_out = 1'b0;
    ready = 1'b0;
    
    case (current_state)
        IDLE: begin
            ready = 1'b1;
            if (data_in != '0) begin
                next_state = ACTIVE;
                counter_rst = 1'b1;
            end
        end
        
        ACTIVE: begin
            counter_en = 1'b1;
            if (counter == 8'hFF) begin
                next_state = WAIT_ACK;
                valid_out = 1'b1;
            end
        end
        
        WAIT_ACK: begin
            valid_out = 1'b1;
            if (/* ack received */) begin
                next_state = IDLE;
            end
        end
    endcase
end

endmodule

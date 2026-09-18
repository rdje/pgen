import q::*;
class c;
  typedef uvm_object_registry#(c, "c") type_id;
  static function type_id get_type();
    return type_id::get();
  endfunction
endclass

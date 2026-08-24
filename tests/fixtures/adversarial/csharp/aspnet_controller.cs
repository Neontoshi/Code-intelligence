// tests/fixtures/adversarial/csharp/aspnet_controller.cs

using Microsoft.AspNetCore.Mvc;
using System.Threading.Tasks;

namespace App.Controllers
{
    [ApiController]
    [Route("api/[controller]")]
    public class OrderController : ControllerBase
    {
        private readonly IOrderService _orderService;

        public OrderController(IOrderService orderService)
        {
            _orderService = orderService;
        }

        // Framework endpoint: looks dead to static AST but called by ASP.NET runtime
        [HttpGet("{id}")]
        public async Task<IActionResult> GetOrder(string id)
        {
            var order = await _orderService.FindByIdAsync(id);
            return Ok(order);
        }

        [HttpPost]
        public async Task<IActionResult> CreateOrder([FromBody] CreateOrderDto dto)
        {
            var id = await _orderService.CreateAsync(dto);
            return CreatedAtAction(nameof(GetOrder), new { id }, id);
        }

        // Truly dead private helper
        private void UnusedInternalMethod()
        {
            // Uncalled logic
        }
    }

    public interface IOrderService
    {
        Task<object> FindByIdAsync(string id);
        Task<string> CreateAsync(CreateOrderDto dto);
    }

    public class CreateOrderDto { }
}

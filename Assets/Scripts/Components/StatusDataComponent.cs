namespace Assets.Scripts.Components.Core
{
    using System.Collections.Generic;

    public class StatusDataComponentCore
    {
        public string Name { get; set; } = null;
        public string Guid { get; set; } = null;
        public string Energy { get; set; } = null;
        public List<string> Dispatchers { get; set; } = null;
        public List<string> DispatchHistory { get; set; } = null;
        public List<string> Receivers { get; set; } = null;
        public Dictionary<string, string> Resources { get; set; } = null;
        public Dictionary<string, string> Info { get; set; } = null;
        public List<Dictionary<uint, string>> Alerts = null;
    }
}
